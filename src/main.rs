#![no_main]
#![no_std]

mod board;
mod boot;
mod clint;
mod collections;
mod cpu;
mod mem;
mod pmp;
mod scheduler;
mod syscall;
mod task;
mod trap;
mod uart;
mod user;

extern "C" fn task_a() -> ! {
    loop {
        user::sys_write("A\n");
        user::sys_yield();
    }
}

extern "C" fn task_b() -> ! {
    loop {
        user::sys_write("B\n");
        user::sys_yield();
    }
}

extern "C" fn task_idle() -> ! {
    loop {
        user::sys_yield();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    unsafe {
        mem::initialize_bss();
        trap::configure_mtvec();
        pmp::allow_all();

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
        cpu::clear_mstatus(cpu::MSTATUS_MPP_MASK);
        cpu::set_mstatus(cpu::MSTATUS_MPIE);
    }

    unsafe { cpu::restore_context(scheduler::start()) }
}
