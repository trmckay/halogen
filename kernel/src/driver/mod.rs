mod sifive_u;
mod virt;

#[cfg(platform = "virt")]
pub use virt::*;

#[cfg(platform = "sifive_u")]
pub use sifive_u::*;

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
