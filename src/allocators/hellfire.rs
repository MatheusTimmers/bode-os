use core::alloc::Layout;
use core::ptr::{self, NonNull};

use super::Allocator;

const USED: usize = 1;
const HEADER: usize = size_of::<Block>();
const WORD: usize = size_of::<usize>();

#[repr(C)]
struct Block {
    next: *mut Block,
    size: usize,
}

pub struct Hellfire {
    first_free: *mut Block,
    last_free: *mut Block,
}

fn align_to_word(size: usize) -> usize {
    size.next_multiple_of(WORD)
}

impl Hellfire {
    pub fn new(heap: &'static mut [u8]) -> Self {
        let len = heap.len();
        let first = heap.as_mut_ptr().cast::<Block>();
        let last = heap.as_mut_ptr().wrapping_add(len - HEADER).cast::<Block>();

        unsafe {
            (*first).next = last;
            (*first).size = align_to_word(len) - 2 * HEADER;
            (*last).next = ptr::null_mut();
            (*last).size = 0;
        }

        Self {
            first_free: first,
            last_free: first,
        }
    }
}

impl Allocator for Hellfire {
    fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        if layout.align() > WORD {
            return None;
        }
        let size = align_to_word(layout.size());

        unsafe {
            let mut p = self.last_free;
            let mut q = p;

            while !(*p).next.is_null() {
                while (*p).size & USED != 0 {
                    p = (*p).next;
                    q = p;
                }
                while (*p).size & USED == 0 && !(*p).next.is_null() {
                    p = (*p).next;
                }
                (*q).size = p.addr() - q.addr() - HEADER;
                (*q).next = p;
                if (*q).size >= size + HEADER {
                    p = q;
                    break;
                }
            }

            if (*p).next.is_null() {
                return None;
            }

            self.last_free = p;
            let rest = (*p).next;
            let new_block = p.cast::<u8>().add(size + HEADER).cast::<Block>();
            (*p).next = new_block;
            (*p).size = size | USED;
            (*new_block).next = rest;
            (*new_block).size = ((*p).size & !USED).wrapping_sub(size).wrapping_sub(HEADER);

            NonNull::new(p.add(1).cast::<u8>())
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, _layout: Layout) {
        unsafe {
            let block = ptr.as_ptr().cast::<Block>().sub(1);
            (*block).size &= !USED;
        }
        self.last_free = self.first_free;
    }
}

#[cfg(test)]
mod tests {
    use core::alloc::Layout;

    use super::Hellfire;
    use crate::allocators::{Allocator, test_heap};

    fn layout(size: usize) -> Layout {
        Layout::from_size_align(size, 4).unwrap()
    }

    #[test]
    fn blocks_do_not_overlap() {
        let mut heap = Hellfire::new(test_heap());
        let a = heap.alloc(layout(100)).unwrap().as_ptr().addr();
        let b = heap.alloc(layout(100)).unwrap().as_ptr().addr();
        assert!(a + 100 <= b || b + 100 <= a);
    }

    #[test]
    fn a_freed_block_with_room_for_a_header_is_reused() {
        let mut heap = Hellfire::new(test_heap());
        let first = heap.alloc(layout(128)).unwrap();
        let _second = heap.alloc(layout(64)).unwrap();
        unsafe { heap.dealloc(first, layout(128)) };
        let third = heap.alloc(layout(64)).unwrap();
        assert_eq!(first, third);
    }

    #[test]
    fn a_freed_block_of_the_exact_size_is_not_reused() {
        let mut heap = Hellfire::new(test_heap());
        let first = heap.alloc(layout(64)).unwrap();
        let _second = heap.alloc(layout(64)).unwrap();
        unsafe { heap.dealloc(first, layout(64)) };
        let third = heap.alloc(layout(64)).unwrap();
        assert_ne!(first, third);
    }

    #[test]
    fn neighbours_are_coalesced_on_demand() {
        let mut heap = Hellfire::new(test_heap());
        let a = heap.alloc(layout(64)).unwrap();
        let b = heap.alloc(layout(64)).unwrap();
        let _c = heap.alloc(layout(64)).unwrap();
        unsafe {
            heap.dealloc(a, layout(64));
            heap.dealloc(b, layout(64));
        }
        let merged = heap.alloc(layout(128)).unwrap();
        assert_eq!(merged, a);
    }

    #[test]
    fn runs_out_and_says_so() {
        let mut heap = Hellfire::new(test_heap());
        assert!(heap.alloc(layout(1 << 20)).is_none());
    }

    #[test]
    fn refuses_alignment_it_cannot_honour() {
        let mut heap = Hellfire::new(test_heap());
        assert!(
            heap.alloc(Layout::from_size_align(16, 64).unwrap())
                .is_none()
        );
    }
}
