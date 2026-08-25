#![no_main]
#![no_std]

use core::{
    arch::naked_asm,
    fmt::{self, Write},
    panic::PanicInfo,
};

const THR: *mut u8 = 0x1000_0000 as *mut u8;
const LSR: *mut u8 = 0x1000_0005 as *mut u8;
const LSR_MASK: u8 = 1 << 5;

pub struct Uart;

#[panic_handler]
#[inline(never)]
fn panic(panic: &PanicInfo<'_>) -> ! {
    let mut uart = Uart;
    let _ = writeln!(uart, "{}", panic);
    loop {}
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
pub unsafe extern "C" fn boot() -> ! {
    naked_asm!("la sp, __stack_top", "j kmain")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmain() -> ! {
    let mut uart = Uart;
    let _ = writeln!(uart, "BODE");
    loop {}
}

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
