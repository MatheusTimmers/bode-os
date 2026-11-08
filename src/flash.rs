use crate::board::{FLASH, flash_chip};

pub fn read(offset: usize, buffer: &mut [u8]) {
    flash_chip::read(FLASH, offset, buffer);
}

pub fn read_u32(offset: usize) -> u32 {
    let mut bytes = [0; 4];
    read(offset, &mut bytes);
    u32::from_le_bytes(bytes)
}
