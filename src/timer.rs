use crate::board::{CLOCK_HZ, MTIME, MTIMECMP};

pub const TICK: u64 = CLOCK_HZ / 10;

pub fn read_mtime() -> u64 {
    read_u64(MTIME)
}

pub fn read_mtimecmp() -> u64 {
    read_u64(MTIMECMP)
}

fn read_u64(addr: usize) -> u64 {
    let low = addr as *const u32;
    let high = (addr + 4) as *const u32;

    loop {
        unsafe {
            let before = high.read_volatile();
            let low_value = low.read_volatile();
            let high_value = high.read_volatile();

            if high_value == before {
                return (high_value as u64) << 32 | (low_value as u64);
            }
        }
    }
}

pub unsafe fn write_mtimecmp(value: u64) {
    let low: *mut u32 = MTIMECMP as *mut u32;
    let high: *mut u32 = (MTIMECMP + 4) as *mut u32;

    unsafe {
        low.write_volatile(0xFFFF_FFFF);
        high.write_volatile((value >> 32) as u32);
        low.write_volatile(value as u32);
    }
}

pub fn schedule_next_tick(delta: u64) {
    let planned = read_mtimecmp() + delta;
    let now = read_mtime();
    let next = if planned <= now { now + delta } else { planned };
    unsafe { write_mtimecmp(next) };
}

pub fn start_schedule_tick(delta: u64) {
    let next = read_mtime() + delta;
    unsafe { write_mtimecmp(next) };
}
