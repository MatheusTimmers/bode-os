use core::arch::{asm, naked_asm};

/// `mie` bit 7 (MTIE): habilita a interrupção de timer de M-mode.
pub const MIE_MTIE: usize = 1 << 7;

/// `mstatus` bit 3 (MIE): chave geral das interrupções em M-mode.
pub const MSTATUS_MIE: usize = 1 << 3;

pub const MSTATUS_MPIE: usize = 1 << 7;
pub const MSTATUS_MPP_MASK: usize = 0b11 << 11;
pub const MSTATUS_MPP_USER: usize = 0b00 << 11;

pub const MCAUSE_INTERRUPT: usize = 1 << 31;
pub const MCAUSE_MACHINE_TIMER: usize = MCAUSE_INTERRUPT | 7;
pub const MCAUSE_ECALL_FROM_USER: usize = 8;

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

/// Carrega os registradores do `Context` em `addr` e entra na task com `mret`.
///
/// # Safety
/// `addr` deve ser o endereço de um `Context` válido da tabela de tasks, com `mepc` e `sp`
/// de uma task de verdade. As interrupções devem estar desligadas (`mstatus.MIE = 0`), e
/// `mstatus.MPP` e `mstatus.MPIE` já devem estar preparados para o `mret`.
#[unsafe(naked)]
pub unsafe extern "C" fn restore_context(addr: usize) -> ! {
    naked_asm!(
        "csrw mscratch, a0",
        "mv t0, a0",
        "lw t1, 0(t0)",
        "csrw mepc, t1",
        ".irp n, 1,2,3,4,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31",
        "lw x\\n, (\\n*4)(t0)",
        ".endr",
        "lw x5, 20(t0)",
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

#[inline]
pub fn read_mstatus() -> usize {
    let value: usize;
    unsafe { asm!("csrr {}, mstatus", out(reg) value, options(nomem, nostack)) };
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

/// # Safety
/// O valor de `addr` deve ser o endereço físico correto deslocado para a direita >> 2
#[inline]
pub unsafe fn write_pmpaddr(index: usize, addr: usize) {
    unsafe {
        match index {
            0 => asm!("csrw pmpaddr0, {}", in(reg) addr, options(nostack)),
            1 => asm!("csrw pmpaddr1, {}", in(reg) addr, options(nostack)),
            2 => asm!("csrw pmpaddr2, {}", in(reg) addr, options(nostack)),
            3 => asm!("csrw pmpaddr3, {}", in(reg) addr, options(nostack)),
            _ => panic!("pmpaddr{} fora de pmpcfg0", index),
        }
    }
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
