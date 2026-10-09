use core::arch::naked_asm;

use crate::{clint, cpu, println, scheduler, syscall, task};

static mut BOOT_CONTEXT: [usize; 32] = [0; 32];

pub fn configure_mtvec() {
    let addr = trap_handler as *const () as usize;
    unsafe {
        cpu::write_mscratch(&raw mut BOOT_CONTEXT as usize);
        cpu::write_mtvec(addr);
    }
}

/// # Safety
/// Chamar só com as interrupções desligadas, de dentro do handler de trap.
unsafe fn switch_to_next_task() {
    match unsafe { scheduler::schedule() } {
        Ok(addr) => unsafe { cpu::write_mscratch(addr) },
        Err(scheduler::Error::NoReadyTask) => {
            panic!("Nenhuma task pronta: deveria vir o idle")
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handler(ctx_addr: usize) {
    let mcause = cpu::read_mcause();
    match mcause {
        cpu::MCAUSE_MACHINE_TIMER => {
            clint::schedule_next_tick(clint::TICK);
            unsafe { switch_to_next_task() };
        }
        cpu::MCAUSE_ECALL_FROM_USER => unsafe {
            let current = scheduler::current();
            debug_assert_eq!(ctx_addr, task::context_addr(current));

            syscall::skip_ecall(current);
            let request = syscall::from_registers(current);

            match request.number {
                syscall::YIELD => {
                    syscall::set_return(current, 0);
                    switch_to_next_task();
                }
                syscall::WRITE => {
                    let written = syscall::write(request.args[0], request.args[1]);
                    syscall::set_return(current, written);
                }
                _ => syscall::set_return(current, syscall::ERROR),
            }
        },
        _ => {
            let mepc = cpu::read_mepc();
            let mtval = cpu::read_mtval();
            let from_user = cpu::read_mstatus() & cpu::MSTATUS_MPP_MASK == cpu::MSTATUS_MPP_USER;

            if from_user {
                unsafe {
                    let current = scheduler::current();
                    println!(
                        "task {} encerrada: mcause: {:#010x}, mepc: {:#010x}, mtval: {:#010x}",
                        current, mcause, mepc, mtval
                    );
                    task::kill(current);
                    switch_to_next_task();
                }
            } else {
                println!(
                    "Exception no kernel: mcause: {:#010x}, mepc: {:#010x}, mtval: {:#010x}",
                    mcause, mepc, mtval
                );
                cpu::halt();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[unsafe(link_section = ".text.trap")]
pub extern "C" fn trap_handler() -> ! {
    naked_asm!(
        // 1. Troca t0 com mscratch.
        // Agora t0 aponta para o Context, e mscratch guarda o t0 original.
        "csrrw t0, mscratch, t0",

        // 2. Salva todos os registradores
        ".irp n, 1,2,3,4,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31",
        "sw x\\n, (\\n*4)(t0)",
        ".endr",

        // 3. Recupera o t0 original e salva no offset 20
        "csrrw t1, mscratch, t0", // t1 recebe o t0 original. mscratch volta a apontar para Context.
        "sw t1, 20(t0)",          // Salva o t0 original na posição x5 do struct

        // 4. Salva o mepc
        "csrr t1, mepc",
        "sw t1, 0(t0)",

        // 5. Recupera o contexto do handler
        "la sp, __stack_top",

        // 6. Chama o handler
        "mv a0, t0",
        "call {h}",

        // 7. Lê o mscratch pro a0, pra passar como parametro
        "csrr a0, mscratch",

        // 8. Restaura o contexto
        "j {r}",
        h = sym handler,
        r = sym crate::cpu::restore_context
    );
}
