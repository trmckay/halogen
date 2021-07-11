/// UART-driver for the module in QEMU virt.
mod uart;

// Everthing exported here, will be rexported in `driver`
// if this module is included.

pub use uart::Uart;
pub use uart::DEV_UART;

/// Initialize the platform devices.
pub fn platform_init() {
    Uart::init();
}
