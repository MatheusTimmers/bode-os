const OUT_OFFSET: usize = 0x0;
const IN_OFFSET: usize = 0x4;
const FLASH_PINS_BYTE: usize = 1;

const CS: u8 = 1 << 0;
const SCK: u8 = 1 << 1;
const MOSI: u8 = 1 << 2;
const MISO: u32 = 1 << 0;

const READ_COMMAND: u8 = 0x03;
const CLOCKS_BEFORE_SELECT: u8 = 8;

pub fn read(base: usize, offset: usize, buffer: &mut [u8]) {
    let pins = (base + OUT_OFFSET + FLASH_PINS_BYTE) as *mut u8;
    let input = (base + IN_OFFSET) as *const u32;
    let address = offset.to_be_bytes();

    unsafe {
        for _ in 0..CLOCKS_BEFORE_SELECT {
            pins.write_volatile(SCK);
            pins.write_volatile(0);
        }
        pins.write_volatile(CS);
        transfer(pins, input, READ_COMMAND);
        transfer(pins, input, address[1]);
        transfer(pins, input, address[2]);
        transfer(pins, input, address[3]);
        for byte in buffer.iter_mut() {
            *byte = transfer(pins, input, 0);
        }
        pins.write_volatile(0);
    }
}

#[inline(never)]
unsafe fn transfer(pins: *mut u8, input: *const u32, out: u8) -> u8 {
    let mut received = 0;

    for bit in (0..8).rev() {
        let mosi = if out & (1 << bit) != 0 { MOSI } else { 0 };

        unsafe {
            pins.write_volatile(CS | mosi);
            input.read_volatile();
            input.read_volatile();
            let miso = input.read_volatile() & MISO != 0;
            pins.write_volatile(CS | mosi | SCK);
            pins.write_volatile(CS | mosi);
            received = (received << 1) | miso as u8;
        }
    }

    received
}
