use core::alloc::Layout;
use core::ptr::NonNull;

use super::Allocator;

const MIN_SHIFT: usize = 4;
const ORDER_COUNT: usize = 13;
const WORD_BITS: usize = usize::BITS as usize;

struct FreeBlock {
    prev: Option<NonNull<FreeBlock>>,
    next: Option<NonNull<FreeBlock>>,
}

pub struct Buddy {
    free: [Option<NonNull<FreeBlock>>; ORDER_COUNT],
    start: usize,
    end: usize,
    base: NonNull<u8>,
    bits: *mut usize,
    bit_offset: [usize; ORDER_COUNT],
}

impl Buddy {
    pub fn new(heap: &'static mut [u8]) -> Self {
        let size = heap.len();
        let base = NonNull::from(heap).cast::<u8>();
        let base_addrs = base.as_ptr().addr();
        let end = base_addrs + size;

        let mut bit_offset = [0; ORDER_COUNT];
        let mut total_bits = 0;
        for (order, offset) in bit_offset.iter_mut().enumerate() {
            *offset = total_bits;
            total_bits += (size >> (MIN_SHIFT + order)) + 1;
        }
        let words = total_bits.div_ceil(WORD_BITS);

        let bits_skip = base.as_ptr().align_offset(align_of::<usize>());
        let bits = base.as_ptr().wrapping_add(bits_skip).cast::<usize>();
        let bits_end = bits_skip + words * size_of::<usize>();

        let start = if bits_end <= size {
            unsafe { bits.write_bytes(0, words) };
            (base_addrs + bits_end)
                .next_multiple_of(1 << MIN_SHIFT)
                .min(end)
        } else {
            end
        };

        let mut buddy = Self {
            base,
            free: [None; ORDER_COUNT],
            start,
            end,
            bits,
            bit_offset,
        };
        buddy.carve_roots();
        buddy
    }

    fn carve_roots(&mut self) {
        let mut addr = self.start;
        while self.end - addr >= 1 << MIN_SHIFT {
            let aligned_shift = addr.trailing_zeros() as usize;
            let fitting_shift = (self.end - addr).ilog2() as usize;
            let shift = aligned_shift
                .min(fitting_shift)
                .min(MIN_SHIFT + ORDER_COUNT - 1);
            self.push(addr, shift - MIN_SHIFT);
            addr += 1 << shift;
        }
    }

    fn block_at(&self, addr: usize) -> NonNull<FreeBlock> {
        let offset = addr - self.base.as_ptr().addr();
        unsafe { self.base.add(offset) }.cast()
    }

    fn bit_index(&self, addr: usize, order: usize) -> usize {
        self.bit_offset[order] + ((addr - self.start) >> (MIN_SHIFT + order))
    }

    fn is_free(&self, addr: usize, order: usize) -> bool {
        let index = self.bit_index(addr, order);
        let word = unsafe { self.bits.add(index / WORD_BITS).read() };
        word & (1 << (index % WORD_BITS)) != 0
    }

    fn mark(&mut self, addr: usize, order: usize, free: bool) {
        let index = self.bit_index(addr, order);
        let word = unsafe { &mut *self.bits.add(index / WORD_BITS) };
        let mask = 1 << (index % WORD_BITS);
        if free {
            *word |= mask;
        } else {
            *word &= !mask;
        }
    }

    fn push(&mut self, addr: usize, order: usize) {
        let block = self.block_at(addr);
        let next = self.free[order];
        unsafe { block.write(FreeBlock { prev: None, next }) };
        if let Some(mut next) = next {
            unsafe { next.as_mut().prev = Some(block) };
        }
        self.free[order] = Some(block);
        self.mark(addr, order, true);
    }

    fn remove(&mut self, addr: usize, order: usize) {
        let block = self.block_at(addr);
        let FreeBlock { prev, next } = unsafe { block.read() };
        match prev {
            Some(mut prev) => unsafe { prev.as_mut().next = next },
            None => self.free[order] = next,
        }
        if let Some(mut next) = next {
            unsafe { next.as_mut().prev = prev };
        }
        self.mark(addr, order, false);
    }
}

impl Allocator for Buddy {
    fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let order = order_of(layout)?;
        let mut found = (order..ORDER_COUNT).find(|&o| self.free[o].is_some())?;
        let block = self.free[found]?;
        let addr = block.as_ptr().addr();
        self.remove(addr, found);

        while found > order {
            found -= 1;
            self.push(addr + block_size(found), found);
        }
        Some(block.cast())
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let Some(mut order) = order_of(layout) else {
            return;
        };
        let mut addr = ptr.as_ptr().addr();

        while order + 1 < ORDER_COUNT {
            let size = block_size(order);
            let buddy = addr ^ size;
            let merged = addr & !size;
            let inside = merged >= self.start && merged + 2 * size <= self.end;
            if !inside || !self.is_free(buddy, order) {
                break;
            }
            self.remove(buddy, order);
            addr = merged;
            order += 1;
        }
        self.push(addr, order);
    }
}

fn block_size(order: usize) -> usize {
    1 << (MIN_SHIFT + order)
}

fn order_of(layout: Layout) -> Option<usize> {
    let needed = layout.size().max(layout.align());
    let at_least_smallest = needed.max(1 << MIN_SHIFT);
    let block_size = at_least_smallest.checked_next_power_of_two()?;
    let exponent = block_size.trailing_zeros() as usize;
    let order = exponent - MIN_SHIFT;

    if order < ORDER_COUNT {
        Some(order)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Buddy;

    contract_tests!(Buddy::new);
    reuse_tests!(Buddy::new);
}
