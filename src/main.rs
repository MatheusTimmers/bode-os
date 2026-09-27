#![no_main]
#![no_std]

mod board;
mod boot;
mod clint;
mod collections;
mod cpu;
mod mem;
mod task;
mod trap;
mod uart;

static mut STACK_A: task::Stack = task::Stack::new();
static mut STACK_B: task::Stack = task::Stack::new();
static mut STACK_IDLE: task::Stack = task::Stack::new();

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
    let idle_ctx;
    unsafe {
        mem::initialize_bss();
        trap::configure_mtvec();

        idle_ctx = match task::task_create(
            task_idle as *const () as usize,
            task::stack_top(&raw const STACK_IDLE),
        ) {
            Some(id) => task::context_addr(id),
            None => panic!("sem vaga para o Idle"),
        };

        if task::task_create(
            task_a as *const () as usize,
            task::stack_top(&raw const STACK_A),
        )
        .is_none()
        {
            println!("sem vaga para a task A");
        }

        if task::task_create(
            task_b as *const () as usize,
            task::stack_top(&raw const STACK_B),
        )
        .is_none()
        {
            println!("sem vaga para a task B");
        }
    };

    println!("Iniciando o Kernel BODE...");

    clint::start_schedule_tick(clint::TICK);
    unsafe {
        cpu::set_mie(cpu::MIE_MTIE);
        cpu::set_mstatus(cpu::MSTATUS_MPP_M | cpu::MSTATUS_MPIE);
    }

    cpu::restore_context(idle_ctx)
}
