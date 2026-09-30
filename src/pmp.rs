use crate::cpu;

pub const READ: usize = 1 << 0;
pub const WRITE: usize = 1 << 1;
pub const EXECUTE: usize = 1 << 2;
pub const TOR: usize = 1 << 3;

const TOP_OF_ADDRESS_SPACE: usize = usize::MAX;

pub fn allow_all() {
    let config = TOR | READ | WRITE | EXECUTE;
    let addr = TOP_OF_ADDRESS_SPACE;

    unsafe {
        cpu::write_pmpaddr0(addr);
        cpu::write_pmpcfg0(config);
    }
}
