mod uart;

pub const DEV_UART: usize = 0x10000000;

pub use uart::Uart;
