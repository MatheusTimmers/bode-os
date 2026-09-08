#![no_main]
#![no_std]

mod boot;
mod clint;
mod cpu;
mod mem;
mod trap;
mod uart;
mod task;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    println!("{}", panic);
    cpu::halt();
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    unsafe {
        mem::initialize_bss();
        trap::configure_mtvec();
        let ctx_addr = task::get_current_context_addr();
        cpu::write_mscratch(ctx_addr);
    };

    println!("Iniciando o Kernel BODE...");

    clint::schedule_next_tick(clint::TICK);
    unsafe {
        cpu::set_mie(cpu::MIE_MTIE);
        cpu::set_mstatus(cpu::MSTATUS_MIE);
    }

    loop {
        cpu::wfi();
    }
}
