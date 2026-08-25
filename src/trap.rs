use crate::{cpu::halt, println};

pub fn configure_mtvec() {
    unsafe {
        let addr = handler as *const () as usize;
        core::arch::asm!("csrw mtvec, {}", in(reg) addr);
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.trap")]
pub fn handler() -> ! {
    let mcause: usize;
    let mepc: usize;
    let mtval: usize;

    unsafe {
        core::arch::asm!("csrr {}, mcause", out(reg) mcause);
        core::arch::asm!("csrr {}, mepc", out(reg) mepc);
        core::arch::asm!("csrr {}, mtval", out(reg) mtval);
    };

    println!("mcause: {:#010x}, mepc: {:#010x}, mtval: {:#010x}", mcause, mepc, mtval);
    halt();
}
