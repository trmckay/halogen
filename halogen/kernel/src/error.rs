#[derive(Debug, Copy, Clone)]
pub enum KernelError {
    HeapAllocationOutOfSpace,
    HeapInvalidFree,
    OutOfVirtualAddresses,
    PageTableAllocation,
    InvalidMapping,
    OutOfPhysicalFrames,
    ExecutableFormat,
    Sbi,
    InvalidSysCall,
}
