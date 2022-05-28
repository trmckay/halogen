//! The `Stack` object is used in trap handlers and kernel threads.

use halogen_common::mem::{alloc::SegmentAllocator, Address, Segment, VirtualAddress};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    error::KernelError,
    kerror,
    log::*,
    mem::{
        paging::{map, Permissions, PAGE_SIZE},
        regions::STACK,
    },
};

lazy_static! {
    // Constructed on first access, so do not access until heap is initialized.
    static ref STACK_ALLOCATOR: Mutex<SegmentAllocator<VirtualAddress>> = {
        Mutex::new(SegmentAllocator::new(*STACK, PAGE_SIZE))
    };
}

/// Kernel stack.
pub struct Stack(Segment<VirtualAddress>);

unsafe impl Sync for Stack {}
unsafe impl Send for Stack {}

impl Stack {
    /// Allocate and map a new stack.
    pub fn new(size: usize) -> Result<Stack, KernelError> {
        let guard_base = match STACK_ALLOCATOR.lock().alloc(size + 2 * PAGE_SIZE) {
            Some(stack) => stack,
            None => {
                return kerror!(
                    KernelError::StackAllocation,
                    kerror!(KernelError::OutOfVirtualAddresses)
                )
                .into()
            }
        };

        let virt_base = guard_base + PAGE_SIZE;

        unsafe {
            if let Err(why) = map(Some(virt_base), None, size, Permissions::ReadWrite) {
                return kerror!(KernelError::StackAllocation, why).into();
            }
        }

        let segment = Segment::from_size(virt_base, size);
        info!("Create stack {}", segment);
        Ok(Stack(segment))
    }

    /// Get a pointer to the top of the stack.
    pub fn top(&self) -> *mut u8 {
        self.0.end.as_mut_ptr()
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        STACK_ALLOCATOR.lock().free(self.0.start)
    }
}
