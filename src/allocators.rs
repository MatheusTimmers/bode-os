use core::alloc::Layout;
use core::ptr::NonNull;

#[cfg(test)]
macro_rules! contract_tests {
    ($new:expr $(, #[$meta:meta])*) => {
        use core::alloc::Layout;
        use $crate::allocators::{test_heap, Allocator, TEST_HEAP_SIZE};

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
