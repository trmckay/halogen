mod uart;

pub const DEV_PLIC: usize = 0xc000000;
pub const DEV_UART0: usize = 0x10010000;
pub const DEV_UART1: usize = 0x10011000;

pub use uart::Uart;
