use core::arch::naked_asm;

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
pub extern "C" fn boot() -> ! {
    naked_asm!("la sp, __stack_top", "j kmain");
}
