use core::arch::asm;

use crate::kernel::syscall;

pub fn sys_yield() {
    unsafe {
        asm!(
            "ecall",
            in("a7") syscall::YIELD,
            lateout("a0") _,
        );
    }
}

pub fn sys_write(bytes: &[u8]) -> isize {
    let ret: usize;
    unsafe {
        asm!(
            "ecall",
            in("a7") syscall::WRITE,
            inlateout("a0") bytes.as_ptr() as usize => ret,
            in("a1") bytes.len(),
        );
    }
    ret as isize
}

pub fn sys_read() -> Option<u8> {
    let ret: usize;
    unsafe {
        asm!(
            "ecall",
            in("a7") syscall::READ,
            lateout("a0") ret,
        );
    }
    if ret == syscall::ERROR {
        return None;
    }
    Some(ret as u8)
}

pub fn sys_exit() -> ! {
    unsafe {
        asm!("ecall", in("a7") syscall::EXIT, options(noreturn));
    }
}

#[cfg(feature = "hw-test")]
pub fn sys_spawn(entry: extern "C" fn(usize, usize) -> !, size: usize) -> Option<usize> {
    let ret: usize;
    unsafe {
        asm!(
            "ecall",
            in("a7") syscall::SPAWN,
            inlateout("a0") entry as usize => ret,
            in("a1") size,
        );
    }
    if ret == syscall::ERROR {
        return None;
    }
    Some(ret)
}
