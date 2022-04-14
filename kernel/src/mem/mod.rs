pub mod heap;
pub mod paging;
pub mod stack;

mod bitmap;
mod frame_alloc;

pub use bitmap::Bitmap;
pub use frame_alloc::{frame_alloc, frame_alloc_init};
pub use heap::{heap_alloc_init, kfree, kmalloc};
pub use paging::*;
pub use stack::{stack_init, Stack, STACK_SIZE};

/// The size of physical memory available to the kernel
///
/// TODO: Parse the FDT instead of hard-coding this
pub const MEMORY_SIZE: usize = 256 * 1024 * 1024;

pub const WORD: usize = 4;
pub const DWORD: usize = 8;
pub const KIB: usize = 1024;
pub const MIB: usize = 1024 * KIB;
pub const GIB: usize = 1024 * MIB;

pub const MMIO_OFFSET: usize = 0xF000_0000;

pub const DEV_TEST: usize = 0x0010_0000;

pub const DEV_PLIC: usize = 0x0C00_0000;
pub const DEV_PLIC_CONTEXT: usize = 0x0C20_0000;

pub const DEV_UART: usize = 0x1000_0000;

/// Evaluates to true of the address is aligned to a mapping level
#[macro_export]
macro_rules! is_mapping_aligned {
    ($addr:expr, $l:expr) => {
        (0 == $addr
            % (match $l {
                $crate::mem::paging::MappingLevel::FourKilobyte => 4096,
                $crate::mem::paging::MappingLevel::TwoMegabyte => 1024 * 1024 * 2,
                $crate::mem::paging::MappingLevel::OneGigabyte => 1024 * 1024 * 1024,
            }))
    };
}
