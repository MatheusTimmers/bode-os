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
        todo!("human: alocar {:?} avançando self.next", layout)
    }

    unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {}
}

#[cfg(test)]
mod tests {
    use super::Bump;

    contract_tests!(Bump::new);
}
