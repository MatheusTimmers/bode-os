unsafe extern "C" {
    static mut __bss: u8;
    static __bss_end: u8;
}

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
