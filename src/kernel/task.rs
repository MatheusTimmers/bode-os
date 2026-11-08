use core::ops::Range;

use crate::kernel::scheduler;
use crate::memory::heap;

pub const MAX_TASKS: usize = 10;
const STACK_SIZE: usize = 1024;

pub const MEPC: usize = 0;
pub const SP: usize = 2;
pub const A0: usize = 10;
pub const A1: usize = 11;
pub const A2: usize = 12;
pub const A7: usize = 17;

#[derive(Debug)]
pub enum Error {
    NoFreeSlot,
    NoMemory,
}

/// # Safety
/// Chamar só com as interrupções desligadas, depois de `heap::init`.
pub unsafe fn spawn_idle(entry: extern "C" fn() -> !) -> Result<usize, Error> {
    let index = unsafe { spawn_with_stack(entry)? };
    unsafe { scheduler::set_idle(index) };
    Ok(index)
}

/// # Safety
/// Chamar só com as interrupções desligadas, com uma região que veio de `heap::alloc_region`.
/// Se der erro, a região continua com quem chamou.
pub unsafe fn spawn_process(
    entry: extern "C" fn(usize, usize) -> !,
    region: Range<usize>,
    args: [usize; 2],
) -> Result<usize, Error> {
    let table = unsafe { table_mut() };

    let index = table.spawn_in(entry as usize, region)?;
    table.slots[index].context.regs[A0] = args[0];
    table.slots[index].context.regs[A1] = args[1];
    unsafe { scheduler::enqueue(index) };

    Ok(index)
}

unsafe fn spawn_with_stack(entry: extern "C" fn() -> !) -> Result<usize, Error> {
    let region = unsafe { heap::alloc_region(STACK_SIZE) }.ok_or(Error::NoMemory)?;
    let table = unsafe { table_mut() };

    let result = table.spawn_in(entry as usize, region.clone());
    if result.is_err() {
        unsafe { heap::free_region(region) };
    }
    result
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn reg(index: usize, reg: usize) -> usize {
    let table = unsafe { table() };
    table.slots[index].context.regs[reg]
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn set_reg(index: usize, reg: usize, value: usize) {
    let table = unsafe { table_mut() };
    table.slots[index].context.regs[reg] = value;
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn is_ready(index: usize) -> bool {
    let table = unsafe { table() };
    table.slots[index].state == State::Ready
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn kill(index: usize) {
    let table = unsafe { table_mut() };
    let task = &mut table.slots[index];

    task.state = State::Free;
    unsafe { heap::free_region(task.region.clone()) };
    task.region = 0..0;
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn region(index: usize) -> Range<usize> {
    let table = unsafe { table() };
    table.slots[index].region.clone()
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn context_addr(index: usize) -> usize {
    let table = unsafe { table() };
    table.context_addr(index)
}

static mut TASK_TABLE: TaskTable = TaskTable::new();

/// # Safety
/// Chamar só com as interrupções desligadas, sem guardar a referência além da função que chamou.
#[allow(clippy::deref_addrof)]
unsafe fn table() -> &'static TaskTable {
    unsafe { &*(&raw const TASK_TABLE) }
}

/// # Safety
/// Chamar só com as interrupções desligadas, sem guardar a referência além da função que chamou.
#[allow(clippy::deref_addrof)]
unsafe fn table_mut() -> &'static mut TaskTable {
    unsafe { &mut *(&raw mut TASK_TABLE) }
}

struct TaskTable {
    slots: [Task; MAX_TASKS],
}

impl TaskTable {
    const fn new() -> Self {
        Self {
            slots: [const { Task::new() }; MAX_TASKS],
        }
    }

    fn find_free_slot(&self) -> Option<usize> {
        for (index, slot) in self.slots.iter().enumerate() {
            if slot.state == State::Free {
                return Some(index);
            }
        }
        None
    }

    fn spawn_in(&mut self, entry: usize, region: Range<usize>) -> Result<usize, Error> {
        let index = self.find_free_slot().ok_or(Error::NoFreeSlot)?;
        let task = &mut self.slots[index];

        task.context = Context::zero();
        task.context.regs[MEPC] = entry;
        task.context.regs[SP] = region.end;
        task.region = region;
        task.state = State::Ready;

        Ok(index)
    }

    fn context_addr(&self, index: usize) -> usize {
        &raw const self.slots[index].context as usize
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Free,
    Ready,
}

struct Task {
    context: Context,
    state: State,
    region: Range<usize>,
}

impl Task {
    const fn new() -> Self {
        Self {
            context: Context::zero(),
            state: State::Free,
            region: 0..0,
        }
    }
}

#[repr(C)]
struct Context {
    regs: [usize; 32],
}

impl Context {
    const fn zero() -> Self {
        Self { regs: [0; 32] }
    }
}
