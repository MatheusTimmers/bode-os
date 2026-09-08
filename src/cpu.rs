use core::arch::asm;

/// `mie` bit 7 (MTIE): habilita a interrupção de timer de M-mode.
pub const MIE_MTIE: usize = 1 << 7;

/// `mstatus` bit 3 (MIE): chave geral das interrupções em M-mode.
pub const MSTATUS_MIE: usize = 1 << 3;

#[inline]
pub fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack)) };
}

pub fn halt() -> ! {
    loop {
        wfi();
    }
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

/// Escreve `mscratch` o endereço de memória do contexto da tarefa vigente.
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
