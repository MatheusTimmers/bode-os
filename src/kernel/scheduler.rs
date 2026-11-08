use crate::kernel::task::{self, MAX_TASKS};
use bode_os::collections::queue::Queue;

#[derive(Debug)]
pub enum Error {
    NoReadyTask,
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn enqueue(index: usize) {
    let scheduler = unsafe { scheduler_mut() };
    scheduler.enqueue(index);
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn set_idle(index: usize) {
    let scheduler = unsafe { scheduler_mut() };
    scheduler.idle = Some(index);
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn start() -> usize {
    let scheduler = unsafe { scheduler_mut() };
    scheduler
        .pop_next()
        .expect("nenhuma task foi criada antes do start")
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn schedule() -> Result<usize, Error> {
    let scheduler = unsafe { scheduler_mut() };

    let current = scheduler.current;
    let is_idle = scheduler.idle == Some(current);
    let ready = unsafe { task::is_ready(current) };
    if !is_idle && ready {
        scheduler.enqueue(current);
    }

    scheduler.pop_next()
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn current() -> usize {
    let scheduler = unsafe { scheduler() };
    scheduler.current
}

static mut SCHEDULER: Scheduler = Scheduler::new();

/// # Safety
/// Chamar só com as interrupções desligadas, sem guardar a referência além da função que chamou.
#[allow(clippy::deref_addrof)]
unsafe fn scheduler() -> &'static Scheduler {
    unsafe { &*(&raw const SCHEDULER) }
}

/// # Safety
/// Chamar só com as interrupções desligadas, sem guardar a referência além da função que chamou.
#[allow(clippy::deref_addrof)]
unsafe fn scheduler_mut() -> &'static mut Scheduler {
    unsafe { &mut *(&raw mut SCHEDULER) }
}

struct Scheduler {
    ready: Queue<usize, { MAX_TASKS + 1 }>,
    current: usize,
    idle: Option<usize>,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            current: 0,
            ready: Queue::new(),
            idle: None,
        }
    }

    fn enqueue(&mut self, index: usize) {
        self.ready
            .push(index)
            .expect("fila de prontas cheia: task duplicada ou fila menor que a tabela");
    }

    fn pop_next(&mut self) -> Result<usize, Error> {
        let next = match self.ready.pop() {
            Some(index) => index,
            None => self.ready_idle()?,
        };
        self.current = next;
        Ok(next)
    }

    fn ready_idle(&self) -> Result<usize, Error> {
        let Some(idle) = self.idle else {
            return Err(Error::NoReadyTask);
        };
        let ready = unsafe { task::is_ready(idle) };
        if !ready {
            return Err(Error::NoReadyTask);
        }
        Ok(idle)
    }
}
