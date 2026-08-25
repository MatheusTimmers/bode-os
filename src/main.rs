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

unsafe extern "C" {
    static mut __bss: u8;
    static __bss_end: u8;
}

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
    naked_asm!("la sp, __stack_top", "j kmain");
}

pub unsafe fn configure_mtvec() {
    unsafe {
        let endereco = handler as *const () as usize;
        core::arch::asm!("csrw mtvec, {}", in(reg) endereco);
    }
}

#[unsafe(no_mangle)]
pub unsafe fn handler() -> ! {
    let mcause: usize;
    let mepc: usize;
    let mtval: usize;

    unsafe {
        core::arch::asm!("csrr {}, mcause", out(reg) mcause);
        core::arch::asm!("csrr {}, mepc", out(reg) mepc);
        core::arch::asm!("csrr {}, mtval", out(reg) mtval);
    };

    let mut uart = Uart;
    let _ = writeln!(uart, "mcause: {:#010x}, mepc: {:#010x}, mtval: {:#010x}", mcause, mepc, mtval);
    loop {};
}

pub unsafe fn initialize_bss() {
    let bss_start = &raw mut __bss;
    let bss_end = &raw const __bss_end;

    let size = bss_end.addr() - bss_start.addr();
    unsafe { bss_start.write_bytes(0, size) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmain() -> ! {
    unsafe {
        initialize_bss();
        configure_mtvec()
    };

    let mut uart = Uart;
    let _ = writeln!(uart, "BODE");
    unsafe { core::arch::asm!("ecall") }
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
