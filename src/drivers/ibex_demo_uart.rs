const RX_OFFSET: usize = 0x0;
const TX_OFFSET: usize = 0x4;
const STATUS_OFFSET: usize = 0x8;
const STATUS_RX_EMPTY: u32 = 1 << 0;
const STATUS_TX_FULL: u32 = 1 << 1;

pub fn write_byte(base: usize, byte: u8) {
    let status = (base + STATUS_OFFSET) as *const u32;
    let tx = (base + TX_OFFSET) as *mut u32;

    unsafe {
        while status.read_volatile() & STATUS_TX_FULL != 0 {}
        tx.write_volatile(byte as u32);
    }
}

pub fn read_byte(base: usize) -> Option<u8> {
    let status = (base + STATUS_OFFSET) as *const u32;
    let rx = (base + RX_OFFSET) as *const u32;

    unsafe {
        let empty = status.read_volatile() & STATUS_RX_EMPTY != 0;
        if empty {
            return None;
        }
        Some(rx.read_volatile() as u8)
    }
}
