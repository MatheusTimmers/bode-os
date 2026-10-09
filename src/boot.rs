use core::arch::naked_asm;

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
pub extern "C" fn boot() -> ! {
    naked_asm!(
        "csrr t0, mhartid",
        "bnez t0, 1f",
        "la sp, __stack_top",
        "j kmain",
        "1:",
        "wfi",
        "j 1b",
    );
}
