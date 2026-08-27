use core::arch::naked_asm;

use crate::{clint, cpu, println};

pub fn configure_mtvec() {
    let addr = trap_handler as *const () as usize;
    unsafe { cpu::write_mtvec(addr) };
}

#[unsafe(no_mangle)]
pub extern "C" fn handler() {
    let mcause = cpu::read_mcause();
    if (mcause >> 31) == 1 {
        clint::schedule_next_tick(clint::TICK);
        println!("TICK");
    } else {
        let mepc = cpu::read_mepc();
        let mtval = cpu::read_mtval();

        println!(
            "mcause: {:#010x}, mepc: {:#010x}, mtval: {:#010x}",
            mcause, mepc, mtval
        );
        cpu::halt();
    }
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[unsafe(link_section = ".text.trap")]
pub extern "C" fn trap_handler() -> ! {
    naked_asm!(
        "addi sp, sp, -64",
        "sw ra, 0(sp)",
        "sw t0, 4(sp)",
        "sw t1, 8(sp)",
        "sw t2, 12(sp)",
        "sw t3, 16(sp)",
        "sw t4, 20(sp)",
        "sw t5, 24(sp)",
        "sw t6, 28(sp)",
        "sw a0, 32(sp)",
        "sw a1, 36(sp)",
        "sw a2, 40(sp)",
        "sw a3, 44(sp)",
        "sw a4, 48(sp)",
        "sw a5, 52(sp)",
        "sw a6, 56(sp)",
        "sw a7, 60(sp)",
        "call {h}",
        "lw ra, 0(sp)",
        "lw t0, 4(sp)",
        "lw t1, 8(sp)",
        "lw t2, 12(sp)",
        "lw t3, 16(sp)",
        "lw t4, 20(sp)",
        "lw t5, 24(sp)",
        "lw t6, 28(sp)",
        "lw a0, 32(sp)",
        "lw a1, 36(sp)",
        "lw a2, 40(sp)",
        "lw a3, 44(sp)",
        "lw a4, 48(sp)",
        "lw a5, 52(sp)",
        "lw a6, 56(sp)",
        "lw a7, 60(sp)",
        "addi sp, sp, 64",
        "mret", h = sym handler
    );
}
