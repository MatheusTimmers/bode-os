use core::fmt::{self, Write};

use crate::board::{UART, uart_chip};

pub struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_bytes(s.as_bytes());
        Ok(())
    }
}

pub fn write_bytes(s: &[u8]) {
    for &byte in s {
        uart_chip::write_byte(UART, byte);
    }
}

pub fn read_byte() -> Option<u8> {
    uart_chip::read_byte(UART)
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
