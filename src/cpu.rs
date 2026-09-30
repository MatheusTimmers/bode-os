use core::arch::{asm, naked_asm};

/// `mie` bit 7 (MTIE): habilita a interrupção de timer de M-mode.
pub const MIE_MTIE: usize = 1 << 7;

/// `mstatus` bit 3 (MIE): chave geral das interrupções em M-mode.
pub const MSTATUS_MIE: usize = 1 << 3;

pub const MSTATUS_MPIE: usize = 1 << 7;
pub const MSTATUS_MPP_MASK: usize = 0b11 << 11;

#[inline]
pub fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack)) };
}

pub fn halt() -> ! {
    unsafe {
        clear_mstatus(MSTATUS_MIE);
        clear_mie(MIE_MTIE);
    }

    loop {
        wfi();
    }
}

#[unsafe(naked)]
pub extern "C" fn restore_context(addr: usize) -> ! {
    naked_asm!(
        "csrw mscratch, x10",
        "mv t0, x10",
        "lw t1, 0(t0)",
        "csrw mepc, t1",
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
        "lw x5,  20(t0)",  // t0 obtém seu valor original de volta
        "mret",
    )
}

#[inline]
pub fn read_mcause() -> usize {
    let value: usize;
    unsafe { asm!("csrr {}, mcause", out(reg) value, options(nomem, nostack)) };
    value
}

#[inline]
pub fn read_mepc() -> usize {
    let value: usize;
    unsafe { asm!("csrr {}, mepc", out(reg) value, options(nomem, nostack)) };
    value
}

#[inline]
pub fn read_mtval() -> usize {
    let value: usize;
    unsafe { asm!("csrr {}, mtval", out(reg) value, options(nomem, nostack)) };
    value
}

/// Liga em `mie` os bits presentes em `mask`, sem tocar nos demais.
///
/// # Safety
/// Habilitar uma interrupção antes de o `mtvec` apontar para um handler
/// válido faz o primeiro trap saltar para lugar nenhum.
#[inline]
pub unsafe fn set_mie(mask: usize) {
    unsafe { asm!("csrs mie, {}", in(reg) mask, options(nomem, nostack)) };
}

/// Liga em `mstatus` os bits presentes em `mask`, sem tocar nos demais.
///
/// # Safety
/// `mstatus` controla o modo de privilégio e a chave geral de interrupções.
#[inline]
pub unsafe fn set_mstatus(mask: usize) {
    unsafe { asm!("csrs mstatus, {}", in(reg) mask, options(nomem, nostack)) };
}

/// Desliga em `mie` os bits presentes em `mask`.
///
/// # Safety
/// Desabilitar uma fonte de interrupção pode impedir o kernel de progredir
/// (o tick do escalonador, por exemplo) sem gerar erro nenhum.
#[inline]
pub unsafe fn clear_mie(mask: usize) {
    unsafe { asm!("csrc mie, {}", in(reg) mask, options(nomem, nostack)) };
}

/// Desliga em `mstatus` os bits presentes em `mask`.
///
/// # Safety
/// Ver `set_mstatus`: os bits deste registrador definem privilégio e estado
/// de interrupção da máquina.
#[inline]
pub unsafe fn clear_mstatus(mask: usize) {
    unsafe { asm!("csrc mstatus, {}", in(reg) mask, options(nomem, nostack)) };
}

/// Escreve `mepc`: o endereço para onde o `mret` vai retornar.
///
/// # Safety
/// `addr` tem de ser um endereço de instrução válido e alcançável. Numa
/// exceção síncrona (`ecall`), retornar para o `mepc` original reexecuta a
/// instrução que trapou, normalmente soma-se o tamanho dela antes.
#[inline]
pub unsafe fn write_mepc(addr: usize) {
    unsafe { asm!("csrw mepc, {}", in(reg) addr, options(nomem, nostack)) };
}

/// Escreve `mtvec`: o endereço para onde o processador salta em todo trap.
///
/// # Safety
/// `addr` deve ser o endereço de um handler de trap válido e alinhado em 4
/// bytes. Os dois bits mais baixos são o campo de modo: se `addr` não for
/// múltiplo de 4, o modo fica inválido e nenhum trap chega ao handler.
#[inline]
pub unsafe fn write_mtvec(addr: usize) {
    unsafe { asm!("csrw mtvec, {}", in(reg) addr, options(nomem, nostack)) };
}

/// Escreve `mscratch`: o endereço de memória do contexto da tarefa vigente.
///
/// # Safety
/// O parâmetro `addr` deve ser um ponteiro alinhado e válido para uma estrutura de
/// contexto.
#[inline]
pub unsafe fn write_mscratch(addr: usize) {
    unsafe { asm!("csrw mscratch, {}", in(reg) addr, options(nomem, nostack)) };
}

#[inline]
pub fn read_mscratch() -> usize {
    let value: usize;
    unsafe { asm!("csrr {}, mscratch", out(reg) value, options(nomem, nostack)) };
    value
}

/// Escreve em `pmpaddr0` os bits do endereço físico para a região PMP.
///
/// # Safety
/// O valor de `addr` deve ser o endereço físico correto deslocado para a direita >> 2
#[inline]
pub unsafe fn write_pmpaddr0(addr: usize) {
    unsafe { asm!("csrw pmpaddr0, {}", in(reg) addr, options(nostack)) };
}

/// Escreve em `pmpcfg0` as permissões (R/W/X), o modo de endereçamento (A) e o bit de trava (L)
/// das regras 0 a 3 do PMP, um byte por regra.
///
/// # Safety
/// Com o bit L ligado, a regra passa a valer também para o M-mode e fica travada até o reset:
/// uma região errada pode tirar do próprio kernel o acesso ao seu código ou aos seus dados.
/// Com L desligado, a configuração só afeta o U-mode.
#[inline]
pub unsafe fn write_pmpcfg0(config: usize) {
    unsafe { asm!("csrw pmpcfg0, {}", in(reg) config, options(nostack)) };
}
