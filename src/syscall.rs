use crate::board::{RAM_END, RAM_START};
use crate::uart;

pub const YIELD: usize = 0;
pub const WRITE: usize = 1;

pub const ERROR: usize = usize::MAX;

pub fn write(ptr: usize, len: usize) -> usize {
    match user_buffer(ptr, len) {
        Some(bytes) => {
            uart::write_bytes(bytes);
            len
        }
        None => ERROR,
    }
}

fn user_buffer(ptr: usize, len: usize) -> Option<&'static [u8]> {
    if len == 0 {
        return Some(&[]);
    }
    let end = ptr.checked_add(len)?;
    if ptr < RAM_START || end > RAM_END {
        return None;
    }
    Some(unsafe { core::slice::from_raw_parts(ptr as *const u8, len) })
}
