use crate::{
    mem::{frame_alloc, frame_free, paging, pte_flags::*, AddressAllocator},
    prelude::*,
};

pub const STACK_SIZE: usize = 256 * 1024;
pub const STACK_REGION_SIZE: usize = 8 * 1024 * 1024;
pub const STACK_COUNT: usize = STACK_REGION_SIZE / STACK_SIZE;

lazy_static! {
    static ref ADDRESS_ALLOCATOR: Mutex<Option<AddressAllocator>> = Mutex::new(None);
}

/// # Safety
///
/// * Region should be available solely for new stack allocations
pub unsafe fn stack_init(start: *mut u8, size: usize) {
    *ADDRESS_ALLOCATOR.lock() = Some(AddressAllocator::new(start as usize, size));
}

#[derive(Debug, Clone)]
pub struct Stack {
    base: *mut u8,
    frames: Vec<usize>,
    size: usize,
}

unsafe impl Sync for Stack {}
unsafe impl Send for Stack {}

impl Stack {
    /// Allocate and map a new stack
    pub fn new(frame_count: usize) -> Option<Stack> {
        // Get a virtual address range for the new stack
        let range = ADDRESS_ALLOCATOR
            .lock()
            .as_mut()?
            .alloc_range(frame_count * paging::L0_PAGE_SIZE)?;

        trace!("Allocate a new stack: {}", range);

        // Allocate and map physical frames
        let frames = (0..frame_count)
            .map(|i| {
                let phys_addr = frame_alloc().expect("failed to allocate frames for stack");
                let virt_addr = range.start + (i * paging::L0_PAGE_SIZE);

                unsafe {
                    paging::map(virt_addr, phys_addr, READ | WRITE | VALID);
                }

                phys_addr
            })
            .collect::<Vec<usize>>();

        // Package it all up
        Some(Stack {
            base: range.start as *mut u8,
            size: frame_count * paging::L0_PAGE_SIZE,
            frames,
        })
    }

    /// Get a pointer to the top of the stack
    pub fn top(&self) -> *mut u8 {
        unsafe { self.base.add(self.size) }
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        trace!("Drop stack @ {:p}", self.base);
        ADDRESS_ALLOCATOR
            .lock()
            .as_mut()
            .expect("dropped a stack, but the address allocator doesn't exist")
            .release_range(self.base as usize);
        self.frames
            .iter()
            .for_each(|&frame| unsafe { frame_free(frame) });
    }
}
