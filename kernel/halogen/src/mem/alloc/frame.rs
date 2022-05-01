use super::AllocatorError;
use crate::{
    is_aligned,
    mem::{AddressRange, L0_PAGE_SIZE},
    prelude::*,
};

static FRAME_ALLOCATOR: Mutex<Option<FrameAllocator<L0_PAGE_SIZE>>> = Mutex::new(None);

/// Intitialize the frame allocator
///
/// # Safety
///
/// `start` should point to a `size` bytes contiguous region of free physical
/// memory
pub unsafe fn frame_alloc_init(start: usize, size: usize) {
    info!(
        "Frame allocator intitialized at physical region {:p}..{:p}",
        start as *const u8,
        (start + size) as *const u8
    );
    *FRAME_ALLOCATOR.lock() = Some(FrameAllocator::new(start, size));
}

/// Allocate a physical frame (the frame is not mapped)
pub fn frame_alloc() -> Option<usize> {
    match FRAME_ALLOCATOR.lock().as_mut().unwrap().alloc() {
        Ok(frame) => Some(frame),
        Err(why) => {
            error!("Failed to allocate physical frame: {}", why);
            None
        }
    }
}

/// Free a frame for later use
///
/// # Safety
///
/// * Must be a valid physical frame
pub unsafe fn frame_free(frame: usize) {
    if let Err(why) = FRAME_ALLOCATOR.lock().as_mut().unwrap().free(frame) {
        error!("Failed to release physical frame: {}", why);
    }
}

#[derive(Debug, Clone)]
struct FreeFrame {
    next: *mut FreeFrame,
}

impl FreeFrame {
    /// Create a new frame with no next
    pub fn new() -> FreeFrame {
        FreeFrame {
            next: core::ptr::null_mut(),
        }
    }

    /// Get a reference to the next frame
    pub fn next(&self) -> Option<&FreeFrame> {
        unsafe { self.next.as_ref() }
    }

    /// Get a mutable reference to the next frame
    pub fn next_mut(&mut self) -> Option<&'static mut FreeFrame> {
        unsafe { self.next.as_mut() }
    }

    pub fn addr(&self) -> usize {
        ptr::addr_of!(*self) as usize
    }
}

/// Physical frame allocator
pub struct FrameAllocator<const B: usize> {
    head: *mut FreeFrame,
    range: AddressRange,
    limit: usize,
}

unsafe impl<const B: usize> Sync for FrameAllocator<B> {}
unsafe impl<const B: usize> Send for FrameAllocator<B> {}

/// Physical frame allocator
impl<const B: usize> FrameAllocator<B> {
    /// Create a frame allocator for a specific arena
    pub unsafe fn new(start: usize, size: usize) -> FrameAllocator<B> {
        debug_assert!(is_aligned!(start, B));
        debug_assert!(is_aligned!(size, B));

        *(start as *mut FreeFrame) = FreeFrame::new();

        FrameAllocator {
            head: start as *mut FreeFrame,
            range: AddressRange::new(start, size),
            limit: 1,
        }
    }

    /// Allocate a new frame or return an error
    fn alloc(&mut self) -> Result<usize, AllocatorError> {
        if self.head.is_null() {
            let limit_addr = self.range.start + (self.limit * B);

            if self.range.contains(limit_addr) {
                self.limit += 1;
                Ok(limit_addr)
            } else {
                Err(AllocatorError::OutOfSpace(1))
            }
        } else {
            unsafe {
                let frame = (*self.head).addr();
                self.head = (*self.head).next;
                Ok(frame)
            }
        }
    }

    /// Free a frame and return an error if this fail
    pub unsafe fn free(&mut self, frame: usize) -> Result<(), AllocatorError> {
        if !is_aligned!(frame, L0_PAGE_SIZE) {
            Err(AllocatorError::InvalidFree(frame))
        } else {
            let old_head = self.head;
            self.head = frame as *mut FreeFrame;
            (*self.head).next = old_head;

            Ok(())
        }
    }
}
