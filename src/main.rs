#![no_main]
#![no_std]

use core::panic::PanicInfo;

mod board;
mod boot;
mod cpu;
mod drivers;
mod flash;
mod heap;
mod interpreter;
mod loader;
mod mem;
mod pmp;
mod scheduler;
mod syscall;
mod task;
mod timer;
mod trap;
mod uart;
mod user;

extern "C" fn task_idle() -> ! {
    loop {
        user::sys_yield();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("{}", info);
    cpu::halt();
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    unsafe {
        mem::initialize_bss();
        mem::plant_stack_canary();
        trap::configure_mtvec();
        pmp::protect_kernel();
        heap::init();
        task::spawn_idle(task_idle).expect("sem vaga para o Idle");
        loader::load_programs();
    };

    println!("Iniciando o Kernel BODE...");

    timer::start_schedule_tick(timer::TICK);
    unsafe {
        cpu::set_mie(cpu::MIE_MTIE);
        cpu::clear_mstatus(cpu::MSTATUS_MPP_MASK);
        cpu::set_mstatus(cpu::MSTATUS_MPIE);
    }

    unsafe {
        let first = scheduler::start();
        pmp::allow_region(task::region(first));
        cpu::restore_context(task::context_addr(first))
    }
}
