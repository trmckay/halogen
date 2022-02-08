pub mod heap;
pub mod paging;
pub mod stack;

mod bitmap;
mod frame_alloc;

pub use bitmap::Bitmap;
pub use frame_alloc::{frame_alloc, frame_alloc_init};
pub use heap::{heap_alloc_init, kfree, kmalloc};
pub use paging::*;
pub use stack::{kstack_alloc, kstack_init};

/// The size of physical memory available to the kernel.
///
/// TODO: Parse the FDT instead of hard-coding this.
pub const MEMORY_SIZE: usize = 256 * 1024 * 1024;

pub const MMIO_DEV_TEST: usize = 0x0010_0000;
pub const INIT_STACK_SIZE: usize = 256 * 1024;

extern "C" {
    pub static KERNEL_SIZE: usize;
    pub static KERNEL_START_PHYS: usize;
    pub static KERNEL_START_VIRT: usize;
    pub static TEXT_END: usize;
    pub static DATA_END: usize;
    pub static INIT_STACK_TOP: usize;
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
