use halogen_lib::mem::MIB;

/// Heap management.
pub mod heap;
/// Addresses and metadata for memory-mapped I/O.
pub mod io;
/// Sv39 implementation.
pub mod paging;
/// Physical frame allocation.
pub mod phys;
/// Kernel address-space layout.
pub mod regions;
/// Allocation of unused virtual addresses.
pub mod virt_alloc;

/// Address spaces.
mod addr_space;
pub use addr_space::*;

/// Stack allocation for kernel threads.
mod stack;
pub use stack::*;

/// The size of physical memory available to the kernel.
pub const MEMORY_SIZE: usize = 256 * MIB;
