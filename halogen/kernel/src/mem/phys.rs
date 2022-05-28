//! Here, the frame allocator manages a virtual mapping of the entire physical
//! memory, less the kernel text and data. All other regions pull their frames
//! from here.
//!
//! After boot, it is assumed that no pages have been freed. So, all memory up
//! to the last frame issued is abandoned. This is fine since any allocated
//! frames are for mapping the kernel image, i.e. permanent. So, a new frame
//! allocator is initialized to manage the region after this, but in virtual
//! space.
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
/// records the virtual offset.
///
/// # Safety
///
/// - The physical memory must be linear-mapped at
///   `halogen::mem::regions::virtual_offset()`.
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

/// Allocate a physical frame.
pub fn alloc() -> Option<(VirtualAddress, PhysicalAddress)> {
    unsafe {
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
}

/// Free a physical frame.
///
/// # Safety
///
/// - `frame` must be aligned.
/// - `frame` must be unused/unmapped.
pub unsafe fn free(frame: PhysicalAddress) {
    let _lock;
    if DO_LOCK {
        _lock = FRAME_ALLOCATOR_MUTEX.lock();
    }
    FRAME_ALLOCATOR.free(frame)
}
