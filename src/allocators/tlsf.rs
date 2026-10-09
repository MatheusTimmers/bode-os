use core::alloc::Layout;
use core::ptr::NonNull;

use super::Allocator;

const SL_SHIFT: usize = 4;
const SL_COUNT: usize = 1 << SL_SHIFT;
const FL_COUNT: usize = 18;
const WORD: usize = size_of::<usize>();
const HEADER: usize = 2 * WORD;
const GRANULE: usize = 2 * WORD;
const MIN_PAYLOAD: usize = 2 * WORD;
const MIN_BLOCK: usize = HEADER + MIN_PAYLOAD;
const FL_SHIFT: usize = SL_SHIFT + GRANULE.trailing_zeros() as usize;
const SMALL: usize = 1 << FL_SHIFT;
const MAX_SIZE: usize = (1 << (FL_SHIFT + FL_COUNT - 1)) - GRANULE;
const FREE: usize = 1;

#[repr(C)]
struct Header {
    prev_phys: Option<NonNull<Header>>,
    size: usize,
    next_free: Option<NonNull<Header>>,
    prev_free: Option<NonNull<Header>>,
}

pub struct Tlsf {
    fl_bitmap: u32,
    sl_bitmap: [u32; FL_COUNT],
    free: [[Option<NonNull<Header>>; SL_COUNT]; FL_COUNT],
}

impl Tlsf {
    pub fn new(heap: &'static mut [u8]) -> Self {
        let mut tlsf = Self {
            fl_bitmap: 0,
            sl_bitmap: [0; FL_COUNT],
            free: [[None; SL_COUNT]; FL_COUNT],
        };

        let size = heap.len();
        let base = NonNull::from(heap).cast::<u8>();
        let skip = base.as_ptr().align_offset(GRANULE);
        let usable = size.saturating_sub(skip) / GRANULE * GRANULE;
        if usable < MIN_BLOCK + HEADER {
            return tlsf;
        }

        let block = unsafe { base.add(skip) }.cast::<Header>();
        set_size(block, (usable - 2 * HEADER).min(MAX_SIZE));
        set_prev_phys(block, None);
        let sentinel = next_phys(block);
        set_size(sentinel, 0);
        set_prev_phys(sentinel, Some(block));

        tlsf.insert(block);
        tlsf
    }

    fn find(&self, fl: usize, sl: usize) -> Option<(usize, usize)> {
        let sl_map = self.sl_bitmap[fl] & (u32::MAX << sl);
        if sl_map != 0 {
            return Some((fl, sl_map.trailing_zeros() as usize));
        }

        let fl_map = self.fl_bitmap & (u32::MAX << (fl + 1));
        if fl_map == 0 {
            return None;
        }
        let fl = fl_map.trailing_zeros() as usize;
        Some((fl, self.sl_bitmap[fl].trailing_zeros() as usize))
    }

    fn insert(&mut self, block: NonNull<Header>) {
        let (fl, sl) = mapping(size(block));
        let next = self.free[fl][sl];
        let header = block.as_ptr();
        unsafe {
            (*header).size |= FREE;
            (*header).prev_free = None;
            (*header).next_free = next;
            if let Some(next) = next {
                (*next.as_ptr()).prev_free = Some(block);
            }
        }
        self.free[fl][sl] = Some(block);
        self.sl_bitmap[fl] |= 1 << sl;
        self.fl_bitmap |= 1 << fl;
    }

    fn remove(&mut self, block: NonNull<Header>) {
        let (fl, sl) = mapping(size(block));
        let header = block.as_ptr();
        unsafe {
            (*header).size &= !FREE;
            let prev = (*header).prev_free;
            let next = (*header).next_free;
            match prev {
                Some(prev) => (*prev.as_ptr()).next_free = next,
                None => self.free[fl][sl] = next,
            }
            if let Some(next) = next {
                (*next.as_ptr()).prev_free = prev;
            }
        }
        if self.free[fl][sl].is_none() {
            self.sl_bitmap[fl] &= !(1 << sl);
            if self.sl_bitmap[fl] == 0 {
                self.fl_bitmap &= !(1 << fl);
            }
        }
    }

    fn align_front(&mut self, block: NonNull<Header>, align: usize) -> NonNull<Header> {
        let start = payload(block).as_ptr().addr();
        let mut aligned = start.next_multiple_of(align);
        if aligned != start && aligned - start < MIN_BLOCK {
            aligned = (start + MIN_BLOCK).next_multiple_of(align);
        }
        let gap = aligned - start;
        if gap == 0 {
            return block;
        }

        let moved = unsafe { block.cast::<u8>().add(gap) }.cast::<Header>();
        set_size(moved, size(block) - gap);
        set_prev_phys(moved, Some(block));
        set_prev_phys(next_phys(moved), Some(moved));
        set_size(block, gap - HEADER);
        self.insert(block);
        moved
    }
}

impl Allocator for Tlsf {
    fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let size = layout
            .size()
            .max(MIN_PAYLOAD)
            .checked_next_multiple_of(GRANULE)?;
        let align = layout.align();
        let request = if align <= GRANULE {
            size
        } else {
            size.checked_add(align + MIN_BLOCK)?
        };
        if request > MAX_SIZE {
            return None;
        }

        let (fl, sl) = mapping(round_up_to_class(request));
        if fl >= FL_COUNT {
            return None;
        }
        let (fl, sl) = self.find(fl, sl)?;
        let mut block = self.free[fl][sl]?;
        self.remove(block);

        if align > GRANULE {
            block = self.align_front(block, align);
        }
        if let Some(rest) = split(block, size) {
            self.insert(rest);
        }
        Some(payload(block))
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, _layout: Layout) {
        let mut block = unsafe { ptr.sub(HEADER) }.cast::<Header>();

        if let Some(prev) = prev_phys(block)
            && is_free(prev)
        {
            self.remove(prev);
            set_size(prev, size(prev) + HEADER + size(block));
            set_prev_phys(next_phys(prev), Some(prev));
            block = prev;
        }

        let next = next_phys(block);
        if is_free(next) {
            self.remove(next);
            set_size(block, size(block) + HEADER + size(next));
            set_prev_phys(next_phys(block), Some(block));
        }

        self.insert(block);
    }
}

fn mapping(size: usize) -> (usize, usize) {
    if size < SMALL {
        (0, size / GRANULE)
    } else {
        let log = size.ilog2() as usize;
        let sl = (size >> (log - SL_SHIFT)) - SL_COUNT;
        (log - FL_SHIFT + 1, sl)
    }
}

fn round_up_to_class(size: usize) -> usize {
    if size < SMALL {
        size
    } else {
        let step = 1 << (size.ilog2() as usize - SL_SHIFT);
        size + step - 1
    }
}

fn split(block: NonNull<Header>, wanted: usize) -> Option<NonNull<Header>> {
    let total = size(block);
    if total < wanted + MIN_BLOCK {
        return None;
    }

    let rest = unsafe { payload(block).add(wanted) }.cast::<Header>();
    set_size(rest, total - wanted - HEADER);
    set_prev_phys(rest, Some(block));
    set_prev_phys(next_phys(rest), Some(rest));
    set_size(block, wanted);
    Some(rest)
}

fn payload(block: NonNull<Header>) -> NonNull<u8> {
    unsafe { block.cast::<u8>().add(HEADER) }
}

fn next_phys(block: NonNull<Header>) -> NonNull<Header> {
    unsafe { payload(block).add(size(block)) }.cast()
}

fn size(block: NonNull<Header>) -> usize {
    unsafe { (*block.as_ptr()).size & !FREE }
}

fn is_free(block: NonNull<Header>) -> bool {
    unsafe { (*block.as_ptr()).size & FREE != 0 }
}

fn prev_phys(block: NonNull<Header>) -> Option<NonNull<Header>> {
    unsafe { (*block.as_ptr()).prev_phys }
}

fn set_size(block: NonNull<Header>, size: usize) {
    unsafe { (*block.as_ptr()).size = size };
}

fn set_prev_phys(block: NonNull<Header>, prev: Option<NonNull<Header>>) {
    unsafe { (*block.as_ptr()).prev_phys = prev };
}

#[cfg(test)]
mod tests {
    use super::Tlsf;

    contract_tests!(Tlsf::new);
    reuse_tests!(Tlsf::new);
}
