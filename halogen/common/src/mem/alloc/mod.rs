mod frame;
pub use frame::*;

mod free_list;
pub use free_list::*;

#[cfg(feature = "alloc")]
mod segment;
#[cfg(feature = "alloc")]
pub use segment::*;

#[derive(Clone, Copy, Debug)]
pub struct AllocatorStats {
    pub bytes_free: usize,
    pub bytes_used: usize,
    pub bytes_overhead: usize,
    pub bytes_total: usize,
    pub blocks_total: usize,
    pub blocks_used: usize,
    pub blocks_free: usize,
}
