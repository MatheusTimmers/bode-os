#![no_main]
#![no_std]

mod boot;
mod clint;
mod cpu;
mod mem;
mod task;
mod trap;
mod uart;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    println!("{}", panic);
    cpu::halt();
}

static mut STACK_A: task::Stack = task::Stack::new();
static mut STACK_B: task::Stack = task::Stack::new();

extern "C" fn task_a() -> ! {
    loop {
        println!("A");
        cpu::wfi();
    }
}

extern "C" fn task_b() -> ! {
    loop {
        println!("B");
        cpu::wfi();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    unsafe {
        mem::initialize_bss();
        trap::configure_mtvec();
        cpu::write_mscratch(task::boot_context_addr());

        if task::task_create(
            task_a as *const () as usize,
            task::stack_top(&raw const STACK_A),
        )
        .is_none()
        {
            println!("sem vaga para a task A");
            cpu::halt();
        }

        if task::task_create(
            task_b as *const () as usize,
            task::stack_top(&raw const STACK_B),
        )
        .is_none()
        {
            println!("sem vaga para a task B");
            cpu::halt();
        }
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
