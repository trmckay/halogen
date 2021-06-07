pub mod uart;

pub use uart::UartDriver;
pub use uart::*;

#[macro_export]
macro_rules! mmio_wr {
    ($a:expr, $d:expr) => {
        ($a as *mut u8).write_volatile($d);
    };
}

#[macro_export]
macro_rules! mmio_rd {
    ($a:expr) => {
        ($a as *mut u8).read_volatile();
    };
}
