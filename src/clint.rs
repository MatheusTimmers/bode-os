use crate::board::CLINT;
use crate::board::CLOCK_HZ;

const MTIME_OFFSET: usize = 0xbff8;
const MTIME_CMP_OFFSET: usize = 0x4000;

pub const TICK: u64 = CLOCK_HZ / 10;

pub fn read_mtime() -> u64 {
    let low: *const u32 = (CLINT + MTIME_OFFSET) as *const u32;
    let high: *const u32 = (CLINT + MTIME_OFFSET + 4) as *const u32;

    let mut low_value: u32;
    let mut high_value: u32;
    let mut aux: u32;

    loop {
        unsafe {
            aux = high.read_volatile();
            low_value = low.read_volatile();
            high_value = high.read_volatile();

            if high_value == aux {
                break;
            }
        }
    }

    (high_value as u64) << 32 | (low_value as u64)
}

pub fn read_mtimecmp() -> u64 {
    let low: *const u32 = (CLINT + MTIME_CMP_OFFSET) as *const u32;
    let high: *const u32 = (CLINT + MTIME_CMP_OFFSET + 4) as *const u32;

    let mut low_value: u32;
    let mut high_value: u32;
    let mut aux: u32;

    loop {
        unsafe {
            aux = high.read_volatile();
            low_value = low.read_volatile();
            high_value = high.read_volatile();

            if high_value == aux {
                break;
            }
        }
    }

    (high_value as u64) << 32 | (low_value as u64)
}

pub unsafe fn write_mtimecmp(value: u64) {
    let low: *mut u32 = (CLINT + MTIME_CMP_OFFSET) as *mut u32;
    let high: *mut u32 = (CLINT + MTIME_CMP_OFFSET + 4) as *mut u32;

    unsafe {
        low.write_volatile(0xFFFF_FFFF);
        high.write_volatile((value >> 32) as u32);
        low.write_volatile(value as u32);
    }
}

pub fn schedule_next_tick(delta: u64) {
    let next = read_mtimecmp() + delta;
    unsafe { write_mtimecmp(next) };
}

pub fn start_schedule_tick(delta: u64) {
    let next = read_mtime() + delta;
    unsafe { write_mtimecmp(next) };
}
