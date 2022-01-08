/// A kernel-mode driver for the QEMU UART module
pub mod uart;

/// MMIO address of the QEMU test device
pub const DEV_TEST: usize = 0x0010_0000;
