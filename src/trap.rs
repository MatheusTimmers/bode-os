use crate::{cpu, println};

pub fn configure_mtvec() {
    let addr = handler as *const () as usize;
    unsafe { cpu::write_mtvec(addr) };
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.trap")]
pub fn handler() -> ! {
    let mcause = cpu::read_mcause();
    let mepc = cpu::read_mepc();
    let mtval = cpu::read_mtval();

    println!("mcause: {:#010x}, mepc: {:#010x}, mtval: {:#010x}", mcause, mepc, mtval);
    cpu::halt();
}
