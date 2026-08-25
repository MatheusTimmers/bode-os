#![no_main]
#![no_std]

mod boot;
mod mem;
mod trap;
mod uart;
mod cpu;

use core::panic::PanicInfo;

use crate::cpu::halt;

#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    println!("{}", panic);
    halt();
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    unsafe {
        mem::initialize_bss();
        trap::configure_mtvec()
    };

    println!("Iniciando o Kernel no BODE...");
    print!("Carregando");
    print!(".");
    print!(".");
    println!(".");
    println!("Carregado");

    halt();
}
