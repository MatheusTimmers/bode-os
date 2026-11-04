#[cfg(not(feature = "ibex"))]
mod virt {
    pub use crate::drivers::ns16550 as uart_chip;

    pub const UART: usize = 0x1000_0000;
    pub const MTIME: usize = 0x0200_bff8;
    pub const MTIMECMP: usize = 0x0200_4000;

    pub const CLOCK_HZ: u64 = 10_000_000;
}

#[cfg(feature = "ibex")]
mod ibex {
    pub use crate::drivers::ibex_demo_uart as uart_chip;

    pub const UART: usize = 0x8000_1000;
    pub const MTIME: usize = 0x8000_2000;
    pub const MTIMECMP: usize = 0x8000_2008;

    pub const CLOCK_HZ: u64 = 27_000_000;
}

#[cfg(feature = "ibex")]
pub use ibex::*;
#[cfg(not(feature = "ibex"))]
pub use virt::*;
