use core::ops::Range;

unsafe extern "C" {
    static mut __bss: u8;
    static __bss_end: u8;
    static __text_start: u8;
    static __text_end: u8;
    static __rodata_end: u8;
    static mut __kernel_stack_bottom: usize;
}

const STACK_CANARY: usize = 0x5AFE_C0DE;

/// Zera a região .bss.
///
/// # Safety
/// Deve ser chamada uma única vez, antes de qualquer acesso a variável global.
pub unsafe fn initialize_bss() {
    let bss_start = &raw mut __bss;
    let bss_end = &raw const __bss_end;

    let size = bss_end.addr() - bss_start.addr();
    unsafe { bss_start.write_bytes(0, size) };
}

pub fn text_start() -> usize {
    (&raw const __text_start).addr()
}

pub fn text_end() -> usize {
    (&raw const __text_end).addr()
}

pub fn rodata_end() -> usize {
    (&raw const __rodata_end).addr()
}

pub fn user_shared() -> Range<usize> {
    text_start()..rodata_end()
}

/// # Safety
/// Chamar uma vez no boot, depois de `initialize_bss`.
pub unsafe fn plant_stack_canary() {
    unsafe { (&raw mut __kernel_stack_bottom).write_volatile(STACK_CANARY) };
}

pub fn stack_canary_intact() -> bool {
    unsafe { (&raw const __kernel_stack_bottom).read_volatile() == STACK_CANARY }
}
