use halogen_common::mem::MIB;

pub mod heap;
pub mod io;
pub mod paging;
pub mod phys;
pub mod regions;
pub mod virt_alloc;

mod addr_space;
pub use addr_space::*;

mod stack;
pub use stack::*;


/// The size of physical memory available to the kernel
pub const MEMORY_SIZE: usize = 256 * MIB;
