//! # Frame allocator structure
//!
//! The frame allocator is used to allocate frames from a contiguous region of
//! memory. The allocator itself has no knowledge of paging, virtual vs.
//! physical, etc. It only manages a region that is accessible in one way or
//! another, returning an address in that region. If this is a virtual address
//! and physical frames are required, this address must be translated.
//!
//! # It's usage here
//!
//! Here, the frame allocator manages a virtual mapping of the entire physical
//! memory, less the kernel text and data. All other regions pull their frames
//! from here.
//!
//! This is the only region guaranteed to be contiguous virtually and
//! physically, so translation can be done with just a single offset saved
//! during bootstrap, rather than walking the page-table.

use core::slice::from_raw_parts_mut;

use halogen_common::mem::{
    alloc::FrameAllocator, Address, PhysicalAddress, Segment, VirtualAddress,
};
use spin::Mutex;

use super::regions::PHYSICAL_BASE;
use crate::mem::{
    paging::{PAGE_SIZE, PAGING_ENABLED as DO_LOCK},
    regions::{virtual_offset, KERNEL_SPACE_START},
};

static mut FRAME_ALLOCATOR_MUTEX: Mutex<()> = Mutex::new(());
static mut FRAME_ALLOCATOR: FrameAllocator<PAGE_SIZE> = FrameAllocator::new_uninit();

/// Intitialize the frame allocator for use in bare-paging mode.
///
/// # Safety
///
/// - The segment should be physical memory not reserved by the kernel image or
///   firmware.
/// - The CPU must not have paging enabled. Call `rebase_virt` when enabling
///   paging.
pub unsafe fn init(segment: Segment<PhysicalAddress>) {
    let slice: &'static mut [[u8; PAGE_SIZE]] =
        from_raw_parts_mut(segment.start.as_mut_ptr(), segment.size() / PAGE_SIZE);
    FRAME_ALLOCATOR.init(slice, 0);
}

/// Rebase the frame allocator to its virtual location. This assumes no physical
/// frames have been freed (so the pointers in the linked list don't have to be
/// updated). It rebases the the frame allocator at the first unused frame and
/// records the virtual offset. `free_offset` is the off
///
/// # Safety
///
/// * `virt_base` must be identity mapped to the original region
pub unsafe fn rebase_virt() {
    // Page allocator's arena's offset into physical memory
    let (frames_used, free_start) = FRAME_ALLOCATOR.boundary().unwrap();
    let free_offset = free_start - PHYSICAL_BASE;
    let free_base_virt = KERNEL_SPACE_START + free_offset;
    let slice: &'static mut [[u8; PAGE_SIZE]] = from_raw_parts_mut(
        free_base_virt.as_mut_ptr(),
        (FRAME_ALLOCATOR.size()) / PAGE_SIZE - frames_used,
    );
    FRAME_ALLOCATOR.init(slice, virtual_offset());
}

/// Allocate a frame using whatever address the frame allocator points to
///
/// After bootstrap, this is the virtual mapping of the physical memory; before
/// it is the physical memory
///
/// # Safety
///
/// * `init` must be called first
pub unsafe fn alloc() -> Option<(VirtualAddress, PhysicalAddress)> {
    let _lock;
    if DO_LOCK {
        _lock = FRAME_ALLOCATOR_MUTEX.lock();
    }
    FRAME_ALLOCATOR.alloc().map(|phys_addr| {
        (
            phys_addr
                .add_offset(FRAME_ALLOCATOR.virt_offset())
                .as_virt(),
            phys_addr,
        )
    })
}

/// Free a frame for later use
///
/// # Safety
///
/// * `frame_alloc_init` must be called first
/// * Must be a valid physical frame
/// * Cannot be called before `rebase_virt` in `enable_paging`
pub unsafe fn free(frame: PhysicalAddress) {
    let _lock;
    if DO_LOCK {
        _lock = FRAME_ALLOCATOR_MUTEX.lock();
    }
    FRAME_ALLOCATOR.free(frame)
}
