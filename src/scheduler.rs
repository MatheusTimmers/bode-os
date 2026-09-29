use crate::collections::queue::Queue;
use crate::task::{self, MAX_TASKS};

#[derive(Debug)]
pub enum Error {
    NoReadyTask,
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn enqueue(index: usize) {
    let scheduler = unsafe { &mut *(&raw mut SCHEDULER) };
    scheduler.enqueue(index);
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn start() -> usize {
    let scheduler = unsafe { &mut *(&raw mut SCHEDULER) };

    let index = scheduler
        .pop_next()
        .expect("nenhuma task foi criada antes do start");

    unsafe { task::context_addr(index) }
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn schedule() -> Result<usize, Error> {
    let scheduler = unsafe { &mut *(&raw mut SCHEDULER) };

    let current = scheduler.current;
    if unsafe { task::is_ready(current) } {
        scheduler.enqueue(current);
    }

    let index = scheduler.pop_next()?;
    Ok(unsafe { task::context_addr(index) })
}

static mut SCHEDULER: Scheduler = Scheduler::new();

struct Scheduler {
    ready: Queue<usize, { MAX_TASKS + 1 }>,
    current: usize,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            current: 0,
            ready: Queue::new(),
        }
    }

    fn enqueue(&mut self, index: usize) {
        self.ready
            .push(index)
            .expect("fila de prontas cheia: task duplicada ou fila menor que a tabela");
    }

    fn pop_next(&mut self) -> Result<usize, Error> {
        let index = self.ready.pop().ok_or(Error::NoReadyTask)?;
        self.current = index;
        Ok(index)
    }
}
