extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};

use super::{frame_alloc, paging, Allocator, FreeListAllocator};
use crate::{is_mapping_aligned, prelude::*};

pub const HEAP_SIZE: usize = 64 * 1024 * 1024;

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator;

lazy_static! {
    static ref HEAP_ALLOCATOR: Mutex<Option<FreeListAllocator>> = Mutex::new(None);
}

/// Initialize the kernel heap
///
/// # Safety
///
/// `start` must point to the virtual address of the heap region and `HEAP_SIZE`
/// bytes of physical frames must be available (both in the virtual address
/// space and physical frames)
pub unsafe fn heap_alloc_init<T>(start: *mut T, size: usize) {
    debug_assert!(is_mapping_aligned!(
        start as usize,
        paging::MappingLevel::FourKilobyte
    ));

    let start = start as *mut u8;

    // Map the heap.
    for i in (0..size).step_by(paging::L0_PAGE_SIZE) {
        let frame = frame_alloc().expect("Out of memory");
        paging::map(
            start.add(i) as usize,
            frame as usize,
            paging::pte_flags::READ | paging::pte_flags::WRITE | paging::pte_flags::VALID,
        );
    }

    info!(
        "Kernel heap located at {:p}..{:p} ({}K)",
        start as *const u8,
        start.add(size),
        size / 1024
    );

    // Initialize the allocator.
    *HEAP_ALLOCATOR.lock() = Some(FreeListAllocator::new(start, size));
}

/// Allocate `size` bytes in the kernel heap
pub fn kmalloc(size: usize) -> Option<*mut u8> {
    trace!("Allocate {} bytes", size);
    match HEAP_ALLOCATOR.lock().as_mut() {
        Some(allocator) => {
            let ptr = allocator.alloc(size);
            match ptr {
                Ok(ptr) => Some(ptr),
                Err(why) => {
                    error!("Heap allocation error: {}", why);
                    None
                }
            }
        }
        None => {
            error!("Heap allocator not initialized");
            None
        }
    }
}

/// Free a heap allocation
///
/// # Safety
///
/// `ptr` must be a pointer returned by `kmalloc()`
pub unsafe fn kfree(ptr: *mut u8) {
    trace!("Free {:p}", ptr);
    match HEAP_ALLOCATOR.lock().as_mut() {
        Some(allocator) => {
            if let Err(why) = allocator.free(ptr) {
                error!("Heap free error: {}", why)
            }
        }
        None => error!("Heap allocator not initialized"),
    }
}

struct GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match kmalloc(layout.size()) {
            Some(ptr) => ptr,
            None => ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        kfree(ptr)
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Global allocator error: size={}", layout.size());
}
