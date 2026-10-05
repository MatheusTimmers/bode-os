use core::arch::asm;

use crate::syscall;

pub fn sys_yield() {
    unsafe {
        asm!(
            "ecall",
            in("a7") syscall::YIELD,
            lateout("a0") _,
        );
    }
}

pub fn sys_write(s: &str) -> isize {
    let ret: usize;
    unsafe {
        asm!(
            "ecall",
            in("a7") syscall::WRITE,
            inlateout("a0") s.as_ptr() as usize => ret,
            in("a1") s.len(),
        );
    }
    ret as isize
}
