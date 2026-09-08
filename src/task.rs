/// Estrutura que representa o contexto de uma tarefa
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub regs: [usize; 32],
}

impl Context {
    pub const fn zero() -> Self {
        Self { regs: [0; 32] }
    }
}

static mut CURRENT_CONTEXT: Context = Context::zero();

pub unsafe fn get_current_context_addr() -> usize {
    core::ptr::addr_of_mut!(CURRENT_CONTEXT) as usize
}
