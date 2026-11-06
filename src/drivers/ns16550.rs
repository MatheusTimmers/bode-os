const RBR_OFFSET: usize = 0x0;
const THR_OFFSET: usize = 0x0;
const LSR_OFFSET: usize = 0x5;
const LSR_DATA_READY: u8 = 1 << 0;
const LSR_THR_EMPTY: u8 = 1 << 5;

pub fn write_byte(base: usize, byte: u8) {
    let lsr = (base + LSR_OFFSET) as *const u8;
    let thr = (base + THR_OFFSET) as *mut u8;

    unsafe {
        while lsr.read_volatile() & LSR_THR_EMPTY == 0 {}
        thr.write_volatile(byte);
    }
}

pub fn read_byte(base: usize) -> Option<u8> {
    let lsr = (base + LSR_OFFSET) as *const u8;
    let rbr = (base + RBR_OFFSET) as *const u8;

    unsafe {
        let ready = lsr.read_volatile() & LSR_DATA_READY != 0;
        if !ready {
            return None;
        }
        Some(rbr.read_volatile())
    }
}
