use core::alloc::Layout;
use core::ptr::NonNull;

use super::Allocator;

pub struct Bump {
    base: NonNull<u8>,
    size: usize,
    next: usize,
}

impl Bump {
    pub fn new(heap: &'static mut [u8]) -> Self {
        let size = heap.len();
        Self {
            base: NonNull::from(heap).cast(),
            size,
            next: 0,
        }
    }
}

impl Allocator for Bump {
    fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let base_heap = self.base.as_ptr().addr();
        let end_heap = base_heap + self.size;
        let current_addrs = base_heap + self.next;

        let start_block = current_addrs.checked_next_multiple_of(layout.align())?;
        let end_block = start_block.checked_add(layout.size())?;

        if end_block <= end_heap {
            self.next = end_block - base_heap;
            let new_addr = self.base.as_ptr().wrapping_add(start_block - base_heap);
            NonNull::new(new_addr)
        } else {
            None
        }
    }

    unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {}
}

#[cfg(test)]
mod tests {
    use super::Bump;

    contract_tests!(Bump::new);
}
