const TX_OFFSET: usize = 0x4;
const STATUS_OFFSET: usize = 0x8;
const STATUS_TX_FULL: u32 = 1 << 1;

pub fn write_byte(base: usize, byte: u8) {
    let status = (base + STATUS_OFFSET) as *const u32;
    let tx = (base + TX_OFFSET) as *mut u32;

    unsafe {
        while status.read_volatile() & STATUS_TX_FULL != 0 {}
        tx.write_volatile(byte as u32);
    }
}
