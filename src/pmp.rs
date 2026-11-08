use core::ops::Range;

use crate::{cpu, mem};

pub const READ: usize = 1 << 0;
pub const WRITE: usize = 1 << 1;
pub const EXECUTE: usize = 1 << 2;
pub const OFF: usize = 0b00 << 3;
pub const TOR: usize = 0b01 << 3;
pub const NAPOT: usize = 0b11 << 3;

const START_RULE: usize = 0;
const TEXT_RULE: usize = 1;
const RODATA_RULE: usize = 2;
const TASK_RULE: usize = 3;

const KERNEL_CONFIG: usize = rule_config(START_RULE, OFF)
    | rule_config(TEXT_RULE, TOR | READ | EXECUTE)
    | rule_config(RODATA_RULE, TOR | READ);

pub fn protect_kernel() {
    unsafe {
        cpu::write_pmpaddr(START_RULE, to_pmpaddr(mem::text_start()));
        cpu::write_pmpaddr(TEXT_RULE, to_pmpaddr(mem::text_end()));
        cpu::write_pmpaddr(RODATA_RULE, to_pmpaddr(mem::rodata_end()));
        cpu::write_pmpcfg0(KERNEL_CONFIG);
    }
}

pub fn allow_region(region: Range<usize>) {
    let config = rule_config(TASK_RULE, NAPOT | READ | WRITE);
    unsafe {
        cpu::write_pmpaddr(TASK_RULE, napot_addr(region));
        cpu::write_pmpcfg0(KERNEL_CONFIG | config);
    }
}

const fn rule_config(rule: usize, config: usize) -> usize {
    let shift = rule * 8;
    config << shift
}

fn to_pmpaddr(addr: usize) -> usize {
    addr >> 2
}

fn napot_addr(region: Range<usize>) -> usize {
    let size = region.end - region.start;

    debug_assert!(size >= 8);
    debug_assert!(size.is_power_of_two());
    debug_assert!(region.start.is_multiple_of(size));

    let mask = (size >> 3) - 1;
    to_pmpaddr(region.start) | mask
}
