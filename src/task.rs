/// Estrutura que representa o contexto de uma tarefa
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub regs: [usize; 32],
}

#[repr(align(16))]
pub struct Stack([u8; 4096]);

impl Stack {
    pub const fn new() -> Self {
        Self([0; 4096])
    }
}

pub unsafe fn stack_top(stack: *const Stack) -> usize {
    stack as usize + core::mem::size_of::<Stack>()
}

impl Context {
    pub const fn zero() -> Self {
        Self { regs: [0; 32] }
    }
}

const MAX_TASKS: usize = 10;
static mut CONTEXTS: [Context; MAX_TASKS] = [Context::zero(); MAX_TASKS];
static mut USED: [bool; MAX_TASKS] = [false; MAX_TASKS];
static mut CURRENT: usize = 0;

pub unsafe fn context_addr(id: usize) -> usize {
    unsafe { &raw mut (CONTEXTS[id]) as usize }
}

unsafe fn next_task() -> Option<usize> {
    unsafe {
        let start = CURRENT + 1;
        for k in 0..MAX_TASKS {
            let i = (start + k) % MAX_TASKS;
            if USED[i] {
                return Some(i);
            }
        }
        None
    }
}

pub unsafe fn next_free_context() -> Option<usize> {
    unsafe {
        for i in 0..MAX_TASKS {
            if !USED[i] {
                return Some(i);
            }
        }
        None
    }
}

pub unsafe fn task_create(entry: usize, sp: usize) -> Option<usize> {
    let index: usize;
    unsafe {
        index = next_free_context()?;

        CONTEXTS[index].regs[0] = entry;
        CONTEXTS[index].regs[2] = sp;
        USED[index] = true;
    }
    Some(index)
}

pub unsafe fn switch_to_next() -> Option<usize> {
    unsafe {
        let next = next_task()?;
        CURRENT = next;
        Some(context_addr(next))
    }
}
