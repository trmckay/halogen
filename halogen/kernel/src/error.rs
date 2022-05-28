use alloc::boxed::Box;

/// Arbitrarily nestable kernel error type. Implements `Into<Result<T, Self>>`.
#[derive(Debug, Clone)]
pub enum KernelError {
    HeapAllocationOutOfSpace(Option<Box<KernelError>>),
    HeapInvalidFree(Option<Box<KernelError>>),
    OutOfVirtualAddresses(Option<Box<KernelError>>),
    PageTableAllocation(Option<Box<KernelError>>),
    InvalidMapping(Option<Box<KernelError>>),
    OutOfPhysicalFrames(Option<Box<KernelError>>),
    ExecutableFormat(Option<Box<KernelError>>),
    Sbi(Option<Box<KernelError>>),
    InvalidSysCall(Option<Box<KernelError>>),
    SchedulerAdd(Option<Box<KernelError>>),
    ThreadCreate(Option<Box<KernelError>>),
    NoSuchThread(Option<Box<KernelError>>),
    StackAllocation(Option<Box<KernelError>>),
    ThreadReap(Option<Box<KernelError>>),
}

/// Construct a new kernel error, optionally with a causing error. This can be
/// chained.
///
/// # Example
///
/// ```
/// let err = kerror!(KernelError::PageTableAllocation, kerror!(KernelError::OutOfPhysicalFrames));
/// ```
#[macro_export]
macro_rules! kerror {
    ($variant:path, $cause:expr) => {
        $variant(Some(alloc::boxed::Box::new($cause)))
    };
    ($variant:path) => {
        $variant(None)
    };
}

impl<T> From<KernelError> for Result<T, KernelError> {
    fn from(err: KernelError) -> Result<T, KernelError> {
        Err(err)
    }
}
