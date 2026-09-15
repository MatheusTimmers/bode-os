use core::arch::naked_asm;

use crate::{clint, cpu, println, task};
use core::panic::PanicInfo;

pub fn configure_mtvec() {
    let addr = trap_handler as *const () as usize;
    unsafe { cpu::write_mtvec(addr) };
}

#[panic_handler]
pub fn panic(panic: &PanicInfo<'_>) -> ! {
    unsafe {
        cpu::clear_mstatus(cpu::MSTATUS_MIE);
        cpu::clear_mie(cpu::MIE_MTIE);
    }

    println!("{}", panic);
    cpu::halt();
}

#[unsafe(no_mangle)]
pub extern "C" fn handler() {
    let mcause = cpu::read_mcause();
    if (mcause >> 31) == 1 {
        clint::schedule_next_tick(clint::TICK);

        unsafe {
            match task::switch_to_next() {
                Some(addr) => cpu::write_mscratch(addr),
                None => println!("nenhuma tarefa pronta"),
            }
        }
    } else {
        let mepc = cpu::read_mepc();
        let mtval = cpu::read_mtval();

        println!(
            "Exception: mcause: {:#010x}, mepc: {:#010x}, mtval: {:#010x}",
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

        // 5. recupera o contexto do handler
        "la sp, __stack_top",

        // 6. Chama o handler
        "call {h}",

        // 7. lê o mscratch pro t0, pra recuperar a base
        "csrr t0, mscratch",

        // 8. Recupera o mepc
        "lw t1, 0(t0)",
        "csrw mepc, t1",

        // 9. Restaura todos os registradores
        "lw x1,  4(t0)",   // ra
        "lw x2,  8(t0)",   // sp
        "lw x3,  12(t0)",  // gp
        "lw x4,  16(t0)",  // tp
        "lw x6,  24(t0)",  // t1
        "lw x7,  28(t0)",  // t2
        "lw x8,  32(t0)",  // s0/fp
        "lw x9,  36(t0)",  // s1
        "lw x10, 40(t0)",  // a0
        "lw x11, 44(t0)",  // a1
        "lw x12, 48(t0)",  // a2
        "lw x13, 52(t0)",  // a3
        "lw x14, 56(t0)",  // a4
        "lw x15, 60(t0)",  // a5
        "lw x16, 64(t0)",  // a6
        "lw x17, 68(t0)",  // a7
        "lw x18, 72(t0)",  // s2
        "lw x19, 76(t0)",  // s3
        "lw x20, 80(t0)",  // s4
        "lw x21, 84(t0)",  // s5
        "lw x22, 88(t0)",  // s6
        "lw x23, 92(t0)",  // s7
        "lw x24, 96(t0)",  // s8
        "lw x25, 100(t0)", // s9
        "lw x26, 104(t0)", // s10
        "lw x27, 108(t0)", // s11
        "lw x28, 112(t0)", // t3
        "lw x29, 116(t0)", // t4
        "lw x30, 120(t0)", // t5
        "lw x31, 124(t0)", // t6

        // 10. Por fim, restaura o t0 original
        "lw x5,  20(t0)",  // t0 obtém seu valor original de volta

        // 11. Retorna do trap de modo máquina
        "mret",
        h = sym handler
    );
}
