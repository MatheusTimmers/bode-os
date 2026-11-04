#[cfg(feature = "ibex")]
pub mod ibex_demo_uart;
#[cfg(not(feature = "ibex"))]
pub mod ns16550;
