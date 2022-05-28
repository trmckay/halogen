use halogen_common::mem::{alloc::SegmentAllocator, VirtualAddress};
use lazy_static::lazy_static;
use spin::Mutex;

use super::regions::VIRT_SPACE;
use crate::{error::KernelError, log::*, mem::paging::PAGE_SIZE};

lazy_static! {
    static ref VIRTUAL_ALLOCATOR: Mutex<SegmentAllocator<VirtualAddress>> = {
        info!("Initialize virtual memory allocator");
        Mutex::new(SegmentAllocator::new(*VIRT_SPACE, PAGE_SIZE))
    };
}

/// Allocate an unused virtual address.
pub fn virt_addr_alloc(size: usize) -> Result<VirtualAddress, KernelError> {
    match VIRTUAL_ALLOCATOR.lock().alloc(size) {
        Some(virt_addr) => Ok(virt_addr),
        None => Err(KernelError::OutOfVirtualAddresses(None)),
    }
}

/// Free an unmapped virtual address.
pub fn virt_addr_free(addr: VirtualAddress) {
    VIRTUAL_ALLOCATOR.lock().free(addr)
}
