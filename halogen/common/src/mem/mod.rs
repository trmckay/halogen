mod addr;
pub use addr::*;

pub mod alloc;

pub const WORD: usize = 4;
pub const DWORD: usize = 8;
pub const KIB: usize = 1024;
pub const MIB: usize = 1024 * KIB;
pub const GIB: usize = 1024 * MIB;
