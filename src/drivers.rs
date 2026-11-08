#[cfg(feature = "ibex")]
pub mod ibex_demo_uart;
#[cfg(not(feature = "ibex"))]
pub mod memory_flash;
#[cfg(not(feature = "ibex"))]
pub mod ns16550;
#[cfg(feature = "ibex")]
pub mod spi_flash_gpio;
