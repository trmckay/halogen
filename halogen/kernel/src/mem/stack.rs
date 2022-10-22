//! The `Stack` object is used in trap handlers and kernel threads.

use halogen_lib::mem::{alloc::SegmentAllocator, Address, Segment, VirtualAddress};
use lazy_static::lazy_static;
use spin::Mutex;

use super::paging::Privilege;
use crate::{
    error::{KernelError, KernelResult},
    kerror,
    mem::{
        paging::{map, Permissions, Scope, PAGE_SIZE},
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
    /// Allocate and map a new stack mapped into kernel space.
    pub fn try_new_kernel(size: usize) -> KernelResult<Stack> {
        let guard_base = STACK_ALLOCATOR
            .lock()
            .alloc(size + 2 * PAGE_SIZE)
            .ok_or_else(|| kerror!(KernelError::OutOfVirtualAddresses))?;

        unsafe {
            Stack::new(
                guard_base.add_offset(PAGE_SIZE as isize),
                size,
                Scope::Global,
                Privilege::Kernel,
            )
        }
    }

    /// Allocate a new stack for use in userspace.
    ///
    /// TODO: Support demand paging.
    ///
    /// # Safety
    ///
    /// - The region described by `base` and `size` must not conflict with any
    ///   existing kernel mappings.
    pub unsafe fn try_new_user(
        segment: Segment<VirtualAddress>,
        init_size: usize,
    ) -> KernelResult<Stack> {
        Stack::new(segment.start, init_size, Scope::Local, Privilege::User)
    }

    /// Create a new stack.
    unsafe fn new(
        base: VirtualAddress,
        size: usize,
        scope: Scope,
        prv: Privilege,
    ) -> KernelResult<Stack> {
        map(Some(base), None, size, Permissions::ReadWrite, scope, prv)?;
        Ok(Stack(Segment::from_size(base, size)))
    }

    /// Get a pointer to the top of the stack.
    pub fn top(&self) -> *mut u8 {
        self.0.end.as_mut_ptr()
    }
}
