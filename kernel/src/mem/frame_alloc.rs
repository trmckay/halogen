use spin::Mutex;

use crate::{is_aligned, mem::L0_PAGE_SIZE};

static FRAME_ALLOCATOR: Mutex<Option<FrameAllocator<L0_PAGE_SIZE>>> = Mutex::new(None);

/// Intitialize the frame allocator.
///
/// # Safety
///
/// `start` should point to a `size` bytes contiguous region of free physical
/// memory.
pub unsafe fn frame_alloc_init(start: usize, size: usize) {
    *FRAME_ALLOCATOR.lock() = Some(FrameAllocator::new(start, size));
}

/// Allocate a physical frame (the frame is not mapped).
pub fn frame_alloc() -> Option<*mut u8> {
    let r = FRAME_ALLOCATOR.lock().as_mut().unwrap().alloc();

    if let Err(why) = &r {
        log::warn!("Failed to allocate frame: {:?}", why);
    }

    match r {
        Ok(ptr) => Some(ptr),
        Err(_) => None,
    }
}

#[derive(Debug)]
pub enum FrameAllocatorError {
    OutOfFrames,
    OutOfBounds(*const u8),
    Misalignment(*const u8),
}

#[derive(Debug, Clone)]
struct FreeFrame {
    next: *mut FreeFrame,
}

impl FreeFrame {
    pub fn new() -> FreeFrame {
        FreeFrame {
            next: core::ptr::null_mut(),
        }
    }

    pub fn next(&self) -> Option<&FreeFrame> {
        unsafe { self.next.as_ref() }
    }

    pub fn next_mut(&mut self) -> Option<&'static mut FreeFrame> {
        unsafe { self.next.as_mut() }
    }
}

pub struct FrameAllocator<const B: usize> {
    start: *mut u8,
    brk: usize,
    head: Option<*mut FreeFrame>,
    size: usize,
}

unsafe impl<const B: usize> Sync for FrameAllocator<B> {}
unsafe impl<const B: usize> Send for FrameAllocator<B> {}

/// Physical frame allocator
impl<const B: usize> FrameAllocator<B> {
    /// Create a frame allocator for a specific arena.
    pub unsafe fn new(start: usize, size: usize) -> FrameAllocator<B> {
        debug_assert!(is_aligned!(start as usize, B));
        debug_assert!(is_aligned!(size, B));

        *(start as *mut FreeFrame) = FreeFrame::new();

        FrameAllocator {
            start: start as *mut u8,
            head: Some(start as *mut FreeFrame),
            brk: 1,
            size,
        }
    }

    fn in_bounds(&self, ptr: *const u8) -> bool {
        (ptr as usize - self.start as usize) < self.size
    }

    pub fn alloc(&mut self) -> Result<*mut u8, FrameAllocatorError> {
        match self.head {
            Some(frame) => {
                let next = unsafe { self.start.add(self.brk * B) };
                self.head = if self.in_bounds(next) {
                    self.brk += 1;
                    Some(next as *mut FreeFrame)
                } else {
                    None
                };
                Ok(frame as *mut u8)
            }
            None => Err(FrameAllocatorError::OutOfFrames),
        }
    }

    pub unsafe fn free(&mut self, ptr: *mut u8, count: usize) -> Result<(), FrameAllocatorError> {
        if !is_aligned!(ptr as usize, B) {
            return Err(FrameAllocatorError::Misalignment(ptr));
        }

        let mut curr = match self.head {
            Some(head) => head,
            None => ptr as *mut FreeFrame,
        };

        for _ in 0..count {
            let next = (curr as usize + B) as *mut FreeFrame;
            (*curr).next = next;
            curr = next;
        }

        Ok(())
    }
}
