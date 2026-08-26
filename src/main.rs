#![no_main]
#![no_std]

mod boot;
mod mem;
mod trap;
mod uart;
mod cpu;
mod clint;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    println!("{}", panic);
    cpu::halt();
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

    cpu::halt();
}
