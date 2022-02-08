use core::arch::asm;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{mem, mem::pte_flags::*};

pub const STACK_SIZE: usize = 256 * 1024;
pub const STACK_REGION_SIZE: usize = 8 * 1024 * 1024;
pub const STACK_COUNT: usize = STACK_REGION_SIZE / STACK_SIZE;

lazy_static! {
    static ref STACK_MANAGER: Mutex<Option<StackManager>> = Mutex::new(None);
}

/// Initialize the kernel stack region.
///
/// # Safety
///
/// `start` must point to the virtual address of the stack region in which
/// `STACK_REGION_SIZE` addresses are available. `STACK_SIZE` bytes of physical
/// frames must be available whenever a `kstack_alloc()` is called.
pub unsafe fn kstack_init(start: usize) {
    *STACK_MANAGER.lock() = Some(StackManager::new(start));

    let sp: usize;
    asm!("mv {}, sp", out(reg) sp, options(nostack));
    let new_stack = kstack_alloc().expect("Failed to allocate kernel stack");
    core::ptr::copy_nonoverlapping(
        (mem::INIT_STACK_TOP - mem::INIT_STACK_SIZE) as *mut u8,
        new_stack.sub(STACK_SIZE),
        mem::INIT_STACK_SIZE,
    );
    let new_sp = new_stack.sub(mem::INIT_STACK_TOP - sp);

    asm!("mv sp, {}", in(reg) new_sp, options(nostack));
}

pub fn kstack_alloc() -> Option<*mut u8> {
    match STACK_MANAGER.lock().as_mut() {
        Some(m) => {
            let ptr = m.alloc()?;
            // TODO: Map these pages on demand.
            for i in (0..STACK_SIZE).step_by(mem::L0_PAGE_SIZE) {
                let frame = mem::frame_alloc()?;
                unsafe { mem::map(ptr.add(i) as usize, frame as usize, READ | WRITE | VALID) };
            }
            Some(unsafe { ptr.add(STACK_SIZE) })
        }
        None => None,
    }
}

pub struct StackManager {
    bitmap: mem::Bitmap<STACK_COUNT, STACK_SIZE>,
}

unsafe impl Sync for StackManager {}
unsafe impl Send for StackManager {}

impl StackManager {
    pub fn new(start: usize) -> StackManager {
        StackManager {
            bitmap: mem::Bitmap::new(start as *mut u8),
        }
    }

    pub fn alloc(&mut self) -> Option<*mut u8> {
        self.bitmap.alloc(1)
    }

    pub fn free(&mut self, ptr: *const u8) {
        self.bitmap.free(ptr)
    }
}
