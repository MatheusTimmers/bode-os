use core::fmt::{self, Write};

const THR: *mut u8 = 0x1000_0000 as *mut u8;
const LSR: *mut u8 = 0x1000_0005 as *mut u8;
const LSR_MASK: u8 = 1 << 5;

pub struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &byte in s.as_bytes() {
            unsafe {
                while LSR.read_volatile() & LSR_MASK == 0b0 {}
                THR.write_volatile(byte);
            }
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    let mut uart = Uart;
    let _ = uart.write_fmt(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::uart::_print(core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", core::format_args!($($arg)*));
    };
}
