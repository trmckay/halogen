use halogen_common::mem::{alloc::SegmentAllocator, Address, Segment, VirtualAddress};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    log::*,
    mem::{
        paging::{map, Permissions, PAGE_SIZE},
        regions::STACK,
    },
};

lazy_static! {
    // Constructed on first access, so do not access until heap is initialized
    static ref STACK_ALLOCATOR: Mutex<SegmentAllocator<VirtualAddress>> = {
        Mutex::new(SegmentAllocator::new(*STACK, PAGE_SIZE))
    };
}

pub struct Stack(Segment<VirtualAddress>);

unsafe impl Sync for Stack {}
unsafe impl Send for Stack {}

impl Stack {
    /// Allocate and map a new stack
    pub fn new(size: usize) -> Option<Stack> {
        let guard_base = STACK_ALLOCATOR.lock().alloc(size + 2 * PAGE_SIZE)?;
        let virt_base = guard_base + PAGE_SIZE;

        unsafe {
            map(Some(virt_base), None, size, Permissions::ReadWrite).ok()?;
        }

        let segment = Segment::from_size(virt_base, size);
        info!("Create stack {}", segment);
        Some(Stack(segment))
    }

    /// Get a pointer to the top of the stack
    pub fn top(&self) -> *mut u8 {
        self.0.end.as_mut_ptr()
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        STACK_ALLOCATOR.lock().free(self.0.start)
    }
}
