use alloc::alloc::{GlobalAlloc, Layout};

use halogen_common::mem::{
    alloc::{AllocatorStats, FreeListAllocator},
    Segment, VirtualAddress, MIB,
};
use spin::Mutex;

use crate::{
    fwprintln, kprintln,
    mem::{
        paging::{map, Permissions},
        regions::HEAP,
    },
};

const START_SIZE: usize = 32 * MIB;
const MIN_ALLOC: usize = 64;

/// The heap allocates space for dynamic data structures using a linked-list
/// allocator. This is a thin wrapper around the `FreeListAllocator` intended to
/// act the `GlobalAlloc` for the `alloc` crate.
#[derive(Debug)]
struct HeapAllocator {
    allocator: Mutex<Option<FreeListAllocator<'static, MIN_ALLOC>>>,
}

impl HeapAllocator {
    /// Create a new heap with no memory to manage yet.
    const fn new_uninit() -> HeapAllocator {
        HeapAllocator {
            allocator: Mutex::new(None),
        }
    }

    /// Initialize the heap with a virtual segment to allocate from. Initially,
    /// back `init_size` bytes with physical frames.
    unsafe fn init(&mut self, segment: Segment<VirtualAddress>, init_size: usize) {
        assert!(init_size <= segment.size());
        map(Some(segment.start), None, init_size, Permissions::ReadWrite).unwrap();
        self.allocator = Mutex::new(Some(FreeListAllocator::new(segment.as_mut_slice())));
    }
}

impl core::fmt::Display for HeapAllocator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.allocator.lock().as_ref() {
            Some(allocator) => write!(f, "Heap:\n{}", allocator),
            None => Ok(()),
        }
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.allocator.lock().as_mut() {
            Some(allocator) => allocator.alloc(layout),
            None => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(allocator) = self.allocator.lock().as_mut() {
            allocator.dealloc(ptr, layout)
        }
    }
}

#[global_allocator]
static mut GLOBAL_ALLOCATOR: HeapAllocator = HeapAllocator::new_uninit();

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    unsafe {
        GLOBAL_ALLOCATOR.allocator.force_unlock();
        match GLOBAL_ALLOCATOR.allocator.lock().as_ref() {
            Some(allocator) => {
                kprintln!("Heap status:\n{:?}", allocator);
            }
            None => fwprintln!("Heap not initialized"),
        }
        panic!("Allocation error: {:?}", layout);
    }
}

/// Get usage stats for the heap.
pub fn stats() -> Option<AllocatorStats> {
    unsafe { Some(GLOBAL_ALLOCATOR.allocator.lock().as_mut()?.stats()) }
}

/// Print the state of the heap to the console.
pub fn dump() {
    unsafe {
        if let Some(allocator) = GLOBAL_ALLOCATOR.allocator.lock().as_ref() {
            kprintln!("Heap state:\n{}", allocator);
        }
    }
}

/// Initialize the heap allocator.
///
/// # Safety
///
/// - Not idempotent.
pub unsafe fn init() {
    GLOBAL_ALLOCATOR.init(*HEAP, START_SIZE)
}
