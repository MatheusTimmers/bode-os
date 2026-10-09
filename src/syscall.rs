use core::ops::Range;

use crate::{mem, task, uart};

pub const YIELD: usize = 0;
pub const WRITE: usize = 1;

pub const ERROR: usize = usize::MAX;

#[derive(Debug)]
pub struct Syscall {
    pub number: usize,
    pub args: [usize; 3],
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn skip_ecall(index: usize) {
    let mepc = unsafe { task::reg(index, task::MEPC) };
    unsafe { task::set_reg(index, task::MEPC, mepc.wrapping_add(4)) };
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn from_registers(index: usize) -> Syscall {
    unsafe {
        Syscall {
            number: task::reg(index, task::A7),
            args: [
                task::reg(index, task::A0),
                task::reg(index, task::A1),
                task::reg(index, task::A2),
            ],
        }
    }
}

/// # Safety
/// Chamar só com as interrupções desligadas.
pub unsafe fn set_return(index: usize, value: usize) {
    unsafe { task::set_reg(index, task::A0, value) };
}

pub fn write(index: usize, ptr: usize, len: usize) -> usize {
    match user_buffer(index, ptr, len) {
        Some(bytes) => {
            uart::write_bytes(bytes);
            len
        }
        None => ERROR,
    }
}

fn user_buffer(index: usize, ptr: usize, len: usize) -> Option<&'static [u8]> {
    if len == 0 {
        return Some(&[]);
    }
    let end = ptr.checked_add(len)?;
    let inside = |region: Range<usize>| region.start <= ptr && end <= region.end;
    if !inside(mem::user_shared()) && !inside(task::stack_range(index)) {
        return None;
    }
    Some(unsafe { core::slice::from_raw_parts(ptr as *const u8, len) })
}
