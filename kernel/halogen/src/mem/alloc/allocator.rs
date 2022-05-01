use core::fmt;

use crate::prelude::*;

pub trait Allocator: Sync + Send {
    /// Allocate space for `count` bytes
    fn alloc_bytes(&mut self, count: usize) -> Result<*mut u8, AllocatorError>;

    /// Allocate space for `count` `T`s
    fn alloc<T>(&mut self, count: usize) -> Result<*mut T, AllocatorError> {
        self.alloc_bytes(count * size_of::<T>())
            .map(|ptr| ptr as *mut T)
    }


    /// Free an allocation
    ///
    /// # Safety
    ///
    /// * `ptr` must be an allocation returned by `self.alloc()`
    unsafe fn free<T>(&mut self, ptr: *mut T) -> Result<(), AllocatorError>;
}

#[derive(Copy, Clone, Debug)]
pub enum AllocatorError {
    InvalidFree(usize),
    OutOfSpace(usize),
    Unspecified,
}

impl fmt::Display for AllocatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocatorError::Unspecified => write!(f, "operation failed"),
            AllocatorError::OutOfSpace(n) => write!(f, "not enough space for {} bytes", n),
            AllocatorError::InvalidFree(p) => write!(f, "{:p} is not a valid pointer", p),
        }
    }
}
