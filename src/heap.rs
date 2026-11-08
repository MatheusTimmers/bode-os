use core::alloc::Layout;
use core::ops::Range;
use core::ptr::NonNull;

use bode_os::allocators::Allocator;
use bode_os::allocators::buddy::Buddy;

const HEAP_SIZE: usize = 8 * 1024;

#[repr(C, align(2048))]
struct Arena([u8; HEAP_SIZE]);

static mut ARENA: Arena = Arena([0; HEAP_SIZE]);
static mut HEAP: Option<Buddy> = None;

/// # Safety
/// Chamar uma vez no boot, depois de `initialize_bss`.
#[allow(clippy::deref_addrof)]
pub unsafe fn init() {
    let arena = unsafe { &mut *(&raw mut ARENA) };
    unsafe { *heap_mut() = Some(Buddy::new(&mut arena.0)) };
}

/// # Safety
/// Chamar só com as interrupções desligadas, depois de `init`.
pub unsafe fn alloc_region(size: usize) -> Option<Range<usize>> {
    let heap = unsafe { heap_mut() }.as_mut()?;
    let layout = Layout::from_size_align(size, size).ok()?;
    let start = heap.alloc(layout)?.as_ptr().addr();
    Some(start..start + size)
}

/// # Safety
/// Chamar só com as interrupções desligadas, com uma região que veio de `alloc_region`.
pub unsafe fn free_region(region: Range<usize>) {
    let Some(heap) = unsafe { heap_mut() }.as_mut() else {
        return;
    };
    let size = region.end - region.start;
    let Ok(layout) = Layout::from_size_align(size, size) else {
        return;
    };
    let Some(start) = NonNull::new(region.start as *mut u8) else {
        return;
    };
    unsafe { heap.dealloc(start, layout) };
}

/// # Safety
/// Chamar só com as interrupções desligadas, sem guardar a referência além da função que chamou.
#[allow(clippy::deref_addrof)]
unsafe fn heap_mut() -> &'static mut Option<Buddy> {
    unsafe { &mut *(&raw mut HEAP) }
}
