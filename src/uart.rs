use core::fmt::{self, Write};

use crate::board::UART;

const THR_OFFSET: usize = 0x0;
const LSR_OFFSET: usize = 0x5;
const LSR_MASK: u8 = 1 << 5;

pub struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let lsr: *mut u8 = (UART + LSR_OFFSET) as *mut u8;
        let thr: *mut u8 = (UART + THR_OFFSET) as *mut u8;

        for &byte in s.as_bytes() {
            unsafe {
                while lsr.read_volatile() & LSR_MASK == 0b0 {}
                thr.write_volatile(byte);
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
        $crate::uart::_print(core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", core::format_args!($($arg)*))
    };
}
