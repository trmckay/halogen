pub mod paging;
pub mod palloc;
pub mod stack;

pub use paging::*;
pub use palloc::{palloc, pfree};

pub const MMIO_DEV_TEST_PHYS: usize = 0x0010_0000;
pub const MMIO_DEV_TEST_VIRT: usize = 0xF010_0000;

extern "C" {
    pub static KERNEL_SIZE: usize;
    pub static KERNEL_START_PHYS: usize;
    pub static PAGING_EN: usize;
    pub static KERNEL_START_VIRT: usize;
}

#[macro_export]
macro_rules! size_of {
    ($t:tt) => {
        (core::mem::size_of::<$t>())
    };
}

#[macro_export]
macro_rules! is_mapping_aligned {
    ($addr:expr, $l:expr) => {
        (0 == $addr
            % (match $l {
                crate::mem::paging::MappingLevel::FourKilobyte => 4096,
                crate::mem::paging::MappingLevel::TwoMegabyte => 1024 * 1024 * 2,
                crate::mem::paging::MappingLevel::OneGigabyte => 1024 * 1024 * 1024,
            }))
    };
}

#[macro_export]
macro_rules! is_aligned {
    ($addr:expr, $d:expr) => {
        (0 == addr % d)
    };
}
