#![no_main]
#![no_std]

use core::panic::PanicInfo;

use arch::{cpu, pmp, trap};
use kernel::{scheduler, task, timer};
use memory::{heap, mem};
use program::loader;

mod arch;
#[cfg(feature = "bench")]
mod bench;
mod board;
mod boot;
mod drivers;
#[cfg(feature = "hw-test")]
mod hwtest;
mod kernel;
mod memory;
mod program;
#[cfg(feature = "switch-bench")]
mod switch_bench;
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

#[cfg_attr(feature = "bench", allow(unreachable_code))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    unsafe {
        mem::initialize_bss();
        mem::plant_stack_canary();
        trap::configure_mtvec();
        pmp::protect_kernel();
        #[cfg(feature = "bench")]
        bench::run();

        heap::init();
        task::spawn_idle(task_idle).expect("sem vaga para o Idle");
        #[cfg(feature = "hw-test")]
        hwtest::spawn_all();

        #[cfg(feature = "switch-bench")]
        switch_bench::prepare();
        if !cfg!(feature = "switch-bench") {
            loader::load_programs();
        }
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
        #[cfg(not(feature = "switch-bench-no-pmp"))]
        pmp::allow_region(task::region(first));
        #[cfg(feature = "switch-bench-no-pmp")]
        pmp::allow_region(switch_bench::shared_block(task::region(first)));
        cpu::restore_context(task::context_addr(first))
    }
}
