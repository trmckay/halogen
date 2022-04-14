use crate::{
    mem,
    mem::{pte_flags::*, Bitmap},
    prelude::*,
};

pub const STACK_SIZE: usize = 256 * 1024;
pub const STACK_REGION_SIZE: usize = 8 * 1024 * 1024;
pub const STACK_COUNT: usize = STACK_REGION_SIZE / STACK_SIZE;

lazy_static! {
    static ref STACK_ALLOCATOR: Mutex<Option<Bitmap<STACK_COUNT, STACK_SIZE>>> = Mutex::new(None);
}

/// Initialize the kernel stack region
///
/// # Safety
///
/// * `start` must point to the virtual address of the stack region in which
///   `STACK_REGION_SIZE` addresses are available
/// * `STACK_SIZE` bytes of physical frames must be available whenever a
///   `kstack_alloc()` is called
pub unsafe fn stack_init(start: usize) {
    *STACK_ALLOCATOR.lock() = Some(Bitmap::new(start as *mut u8));
}

#[derive(Debug, Clone)]
pub struct Stack {
    base: *mut u8,
    size: usize,
}

unsafe impl Sync for Stack {}
unsafe impl Send for Stack {}

impl Stack {
    /// Allocate and map a new stack
    pub fn new() -> Option<Stack> {
        let base = STACK_ALLOCATOR.lock().as_mut().and_then(|sa| sa.alloc(1))?;

        for virt_addr in
            (base as usize..(base as usize + STACK_SIZE)).step_by(mem::paging::L0_PAGE_SIZE)
        {
            let phys_addr =
                mem::frame_alloc().expect("Failed to allocate physical frame for stack") as usize;
            unsafe { mem::paging::map(virt_addr, phys_addr, READ | WRITE | VALID) };
        }

        Some(Stack {
            base,
            size: STACK_SIZE,
        })
    }

    /// Get a pointer to the top of the stack
    pub fn top(&self) -> *mut u8 {
        unsafe { self.base.add(self.size) }
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        // TODO: Unmap the stack
        STACK_ALLOCATOR
            .lock()
            .as_mut()
            .expect("stack allocator uninitialized")
            .free(self.base);
    }
}
