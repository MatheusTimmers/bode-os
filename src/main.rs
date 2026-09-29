#![no_main]
#![no_std]

mod board;
mod boot;
mod clint;
mod collections;
mod cpu;
mod mem;
mod scheduler;
mod task;
mod trap;
mod uart;

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

extern "C" fn task_idle() -> ! {
    loop {
        cpu::wfi();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    unsafe {
        mem::initialize_bss();
        trap::configure_mtvec();

        if task::spawn(task_a).is_err() {
            println!("sem vaga para a task A");
        }

        if task::spawn(task_b).is_err() {
            println!("sem vaga para a task B");
        }

        task::spawn(task_idle).expect("sem vaga para o Idle");
    };

    println!("Iniciando o Kernel BODE...");

    clint::start_schedule_tick(clint::TICK);
    unsafe {
        cpu::set_mie(cpu::MIE_MTIE);
        cpu::set_mstatus(cpu::MSTATUS_MPP_M | cpu::MSTATUS_MPIE);
    }

    cpu::restore_context(unsafe { scheduler::start() })
}
