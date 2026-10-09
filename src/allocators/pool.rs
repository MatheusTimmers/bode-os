use core::alloc::Layout;
use core::ptr::NonNull;

use super::Allocator;

pub struct Pool;

impl Pool {
    pub fn new(_heap: &'static mut [u8]) -> Self {
        todo!()
    }
}

impl Allocator for Pool {
    fn alloc(&mut self, _layout: Layout) -> Option<NonNull<u8>> {
        todo!()
    }

    unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::Pool;

    contract_tests!(Pool::new, #[ignore = "pool ainda não implementado"]);
}
