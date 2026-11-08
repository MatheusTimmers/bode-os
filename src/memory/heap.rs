use core::alloc::Layout;
use core::ops::Range;
use core::ptr::NonNull;

use bode_os::allocators::Allocator;
use bode_os::allocators::buddy::Buddy;

use crate::memory::mem;

static mut HEAP: Option<Buddy> = None;

#[cfg(feature = "bench")]
const BENCH_ARENA_SIZE: usize = 8 * 1024;

/// # Safety
/// Chamar só antes de `init`, e sem usar a arena devolvida por uma chamada anterior.
#[cfg(feature = "bench")]
pub unsafe fn fresh_arena() -> &'static mut [u8] {
    let arena = unsafe { arena() };
    let arena = &mut arena[..BENCH_ARENA_SIZE];
    arena.fill(0);
    arena
}

/// # Safety
/// Chamar uma vez no boot, depois de `initialize_bss`.
pub unsafe fn init() {
    unsafe { *heap_mut() = Some(Buddy::new(arena())) };
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

unsafe fn arena() -> &'static mut [u8] {
    let range = mem::heap_range();
    let size = range.end - range.start;
    unsafe { core::slice::from_raw_parts_mut(range.start as *mut u8, size) }
}

/// # Safety
/// Chamar só com as interrupções desligadas, sem guardar a referência além da função que chamou.
#[allow(clippy::deref_addrof)]
unsafe fn heap_mut() -> &'static mut Option<Buddy> {
    unsafe { &mut *(&raw mut HEAP) }
}
