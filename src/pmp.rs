use core::ops::Range;

use crate::{cpu, mem};

pub const READ: usize = 1 << 0;
pub const WRITE: usize = 1 << 1;
pub const EXECUTE: usize = 1 << 2;
pub const OFF: usize = 0b00 << 3;
pub const TOR: usize = 0b01 << 3;
pub const NAPOT: usize = 0b11 << 3;

const STACK_RULE: usize = 3;
const KERNEL_CONFIG: usize = OFF | (TOR | READ | EXECUTE) << 8 | (TOR | READ) << 16;

pub fn protect_kernel() {
    let addrs = [mem::text_start(), mem::text_end(), mem::rodata_end()];

    for (index, addr) in addrs.into_iter().enumerate() {
        unsafe { cpu::write_pmpaddr(index, addr >> 2) };
    }

    unsafe { cpu::write_pmpcfg0(KERNEL_CONFIG) };
}

pub fn allow_stack(stack: Range<usize>) {
    unsafe {
        cpu::write_pmpaddr(STACK_RULE, napot_addr(stack));
        cpu::write_pmpcfg0(KERNEL_CONFIG | (NAPOT | READ | WRITE) << (STACK_RULE * 8));
    }
}

fn napot_addr(region: Range<usize>) -> usize {
    let size = region.end - region.start;

    debug_assert!(size >= 8);
    debug_assert!(size.is_power_of_two());
    debug_assert!(region.start.is_multiple_of(size));

    (region.start >> 2) | ((size >> 3) - 1)
}
