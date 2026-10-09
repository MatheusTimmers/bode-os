use core::alloc::Layout;
use core::ptr::NonNull;

use super::Allocator;

pub struct Tlsf;

impl Tlsf {
    pub fn new(_heap: &'static mut [u8]) -> Self {
        todo!()
    }
}

impl Allocator for Tlsf {
    fn alloc(&mut self, _layout: Layout) -> Option<NonNull<u8>> {
        todo!()
    }

    unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::Tlsf;

    contract_tests!(Tlsf::new, #[ignore = "tlsf ainda não implementado"]);
}
