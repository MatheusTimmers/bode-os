use core::arch::asm;

pub fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack)) };
}

pub fn halt() -> ! {
    loop { wfi(); }
}
