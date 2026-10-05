use core::arch::naked_asm;

use crate::{clint, cpu, println, scheduler, syscall, task};
use core::panic::PanicInfo;

pub fn configure_mtvec() {
    let addr = trap_handler as *const () as usize;
    unsafe { cpu::write_mtvec(addr) };
}

#[panic_handler]
pub fn panic(panic: &PanicInfo<'_>) -> ! {
    println!("{}", panic);

    unsafe {
        cpu::clear_mstatus(cpu::MSTATUS_MIE);
        cpu::clear_mie(cpu::MIE_MTIE);
    }

    cpu::halt();
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

            task::skip_ecall(current);
            let request = task::syscall_from_registers(current);

            match request.number {
                syscall::YIELD => {
                    task::set_syscall_return(current, 0);
                    switch_to_next_task();
                }
                syscall::WRITE => {
                    let written = syscall::write(request.args[0], request.args[1]);
                    task::set_syscall_return(current, written);
                }
                _ => task::set_syscall_return(current, syscall::ERROR),
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
        "sw x1,  4(t0)",   // ra
        "sw x2,  8(t0)",   // sp
        "sw x3,  12(t0)",  // gp
        "sw x4,  16(t0)",  // tp
        // x5 (t0) será salvo mais tarde
        "sw x6,  24(t0)",  // t1
        "sw x7,  28(t0)",  // t2
        "sw x8,  32(t0)",  // s0/fp
        "sw x9,  36(t0)",  // s1
        "sw x10, 40(t0)",  // a0
        "sw x11, 44(t0)",  // a1
        "sw x12, 48(t0)",  // a2
        "sw x13, 52(t0)",  // a3
        "sw x14, 56(t0)",  // a4
        "sw x15, 60(t0)",  // a5
        "sw x16, 64(t0)",  // a6
        "sw x17, 68(t0)",  // a7
        "sw x18, 72(t0)",  // s2
        "sw x19, 76(t0)",  // s3
        "sw x20, 80(t0)",  // s4
        "sw x21, 84(t0)",  // s5
        "sw x22, 88(t0)",  // s6
        "sw x23, 92(t0)",  // s7
        "sw x24, 96(t0)",  // s8
        "sw x25, 100(t0)", // s9
        "sw x26, 104(t0)", // s10
        "sw x27, 108(t0)", // s11
        "sw x28, 112(t0)", // t3
        "sw x29, 116(t0)", // t4
        "sw x30, 120(t0)", // t5
        "sw x31, 124(t0)", // t6

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
