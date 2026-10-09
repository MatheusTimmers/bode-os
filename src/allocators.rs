use core::alloc::Layout;
use core::ptr::NonNull;

#[cfg(test)]
macro_rules! contract_tests {
    ($new:expr $(, #[$meta:meta])*) => {
        use core::alloc::Layout;
        use $crate::allocators::{test_heap, unaligned_test_heap, Allocator, TEST_HEAP_SIZE};

        fn layout(size: usize, align: usize) -> Layout {
            Layout::from_size_align(size, align).unwrap()
        }

        #[test]
        $(#[$meta])*
        fn respects_alignment() {
            let mut heap = ($new)(test_heap());
            for align in [1, 2, 4, 8, 16, 64] {
                let ptr = heap.alloc(layout(3, align)).unwrap();
                assert!(ptr.as_ptr().addr().is_multiple_of(align));
            }
        }

        #[test]
        $(#[$meta])*
        fn live_blocks_do_not_overlap() {
            let mut heap = ($new)(test_heap());
            let a = heap.alloc(layout(100, 4)).unwrap().as_ptr().addr();
            let b = heap.alloc(layout(100, 4)).unwrap().as_ptr().addr();
            assert!(a + 100 <= b || b + 100 <= a);
        }

        #[test]
        $(#[$meta])*
        fn keeps_what_was_written() {
            let mut heap = ($new)(test_heap());
            let a = heap.alloc(layout(64, 4)).unwrap().as_ptr();
            let b = heap.alloc(layout(64, 4)).unwrap().as_ptr();
            unsafe {
                a.write_bytes(0xAA, 64);
                b.write_bytes(0x55, 64);
                let a = core::slice::from_raw_parts(a, 64);
                assert!(a.iter().all(|&byte| byte == 0xAA));
            }
        }

        #[test]
        $(#[$meta])*
        fn refuses_more_than_the_heap() {
            let mut heap = ($new)(test_heap());
            assert!(heap.alloc(layout(TEST_HEAP_SIZE + 1, 4)).is_none());
        }

        #[test]
        $(#[$meta])*
        fn respects_alignment_on_unaligned_heap() {
            let mut heap = ($new)(unaligned_test_heap());
            for align in [1, 2, 4, 8, 16, 64] {
                let ptr = heap.alloc(layout(3, align)).unwrap();
                assert!(ptr.as_ptr().addr().is_multiple_of(align));
            }
        }

        #[test]
        $(#[$meta])*
        fn runs_out_and_says_so() {
            let mut heap = ($new)(test_heap());
            let mut count = 0;
            while heap.alloc(layout(1024, 8)).is_some() {
                count += 1;
                assert!(count * 1024 <= TEST_HEAP_SIZE, "entregou mais do que a heap tem");
            }
            assert!(count > 0);
        }
    };
}

#[cfg(test)]
macro_rules! reuse_tests {
    ($new:expr $(, #[$meta:meta])*) => {
        mod reuse {
            use super::*;
            use core::alloc::Layout;
            use $crate::allocators::{test_heap, Allocator};

            fn layout(size: usize, align: usize) -> Layout {
                Layout::from_size_align(size, align).unwrap()
            }

            #[test]
            $(#[$meta])*
            fn freeing_everything_gives_it_all_back() {
                let mut heap = ($new)(test_heap());
                let block = layout(1024, 8);
                let mut first = Vec::new();
                while let Some(ptr) = heap.alloc(block) {
                    first.push(ptr);
                }
                for ptr in first.iter().rev() {
                    unsafe { heap.dealloc(*ptr, block) };
                }
                let mut second = 0;
                while heap.alloc(block).is_some() {
                    second += 1;
                }
                assert_eq!(second, first.len());
            }

            #[test]
            $(#[$meta])*
            fn a_freed_hole_is_reused() {
                let mut heap = ($new)(test_heap());
                let block = layout(256, 8);
                let mut live = Vec::new();
                while let Some(ptr) = heap.alloc(block) {
                    live.push(ptr);
                }
                let hole = live.remove(live.len() / 2);
                unsafe { heap.dealloc(hole, block) };
                assert!(heap.alloc(block).is_some());
            }
        }
    };
}

pub mod buddy;
pub mod bump;
pub mod pool;
pub mod tlsf;

pub trait Allocator {
    fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>>;

    /// # Safety
    /// `ptr` precisa ter vindo de `alloc` deste mesmo alocador, com o mesmo `layout`,
    /// e não pode ter sido liberado antes.
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout);
}

#[cfg(test)]
pub(crate) const TEST_HEAP_SIZE: usize = 64 * 1024;

#[cfg(test)]
pub(crate) fn test_heap() -> &'static mut [u8] {
    #[repr(C, align(65536))]
    struct Arena([u8; TEST_HEAP_SIZE]);

    &mut Box::leak(Box::new(Arena([0; TEST_HEAP_SIZE]))).0
}

#[cfg(test)]
pub(crate) fn unaligned_test_heap() -> &'static mut [u8] {
    &mut test_heap()[1..]
}
