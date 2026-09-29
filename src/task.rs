use crate::scheduler;

pub const MAX_TASKS: usize = 10;
const STACK_SIZE: usize = 4096;

#[derive(Debug)]
pub enum Error {
    NoFreeSlot,
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn spawn(entry: extern "C" fn() -> !) -> Result<usize, Error> {
    let table = unsafe { &mut *(&raw mut TASK_TABLE) };

    let index = table.spawn(entry)?;
    unsafe { scheduler::enqueue(index) };

    Ok(index)
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn is_ready(index: usize) -> bool {
    let table = unsafe { &*(&raw const TASK_TABLE) };
    table.slots[index].state == State::Ready
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn context_addr(index: usize) -> usize {
    let table = unsafe { &mut *(&raw mut TASK_TABLE) };
    table.context_addr(index)
}

static mut TASK_TABLE: TaskTable = TaskTable::new();

struct TaskTable {
    slots: [Task; MAX_TASKS],
}

impl TaskTable {
    const fn new() -> Self {
        Self {
            slots: [const { Task::new() }; MAX_TASKS],
        }
    }

    fn stack_top(&self, index: usize) -> usize {
        &raw const self.slots[index].stack as usize + core::mem::size_of::<Stack>()
    }

    fn find_free_slot(&self) -> Option<usize> {
        (0..MAX_TASKS).find(|&i| self.slots[i].state == State::Free)
    }

    fn spawn(&mut self, entry: extern "C" fn() -> !) -> Result<usize, Error> {
        let index = self.find_free_slot().ok_or(Error::NoFreeSlot)?;

        self.slots[index].context = Context::zero();
        self.slots[index].context.regs[Context::MEPC] = entry as usize;
        self.slots[index].context.regs[Context::SP] = self.stack_top(index);
        self.slots[index].state = State::Ready;

        Ok(index)
    }

    fn context_addr(&mut self, index: usize) -> usize {
        &raw mut (self.slots[index].context) as usize
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Free,
    Ready,
}

struct Task {
    context: Context,
    stack: Stack,
    state: State,
}

impl Task {
    const fn new() -> Self {
        Self {
            context: Context::zero(),
            stack: Stack::new(),
            state: State::Free,
        }
    }
}

#[repr(C)]
struct Context {
    regs: [usize; 32],
}

impl Context {
    const MEPC: usize = 0;
    const SP: usize = 2;

    const fn zero() -> Self {
        Self { regs: [0; 32] }
    }
}

#[repr(align(16))]
struct Stack([u8; STACK_SIZE]);

impl Stack {
    const fn new() -> Self {
        Self([0; STACK_SIZE])
    }
}
