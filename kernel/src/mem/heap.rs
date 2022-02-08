extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
pub use alloc::{boxed::*, vec, vec::*};
use core::{
    fmt,
    iter::successors,
    mem::size_of,
    ptr::{addr_of, addr_of_mut, null_mut},
};

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{is_mapping_aligned, mem, mem::paging};

pub const HEAP_SIZE: usize = 8 * 1024 * 1024;

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator;

lazy_static! {
    static ref HEAP_ALLOCATOR: Mutex<Option<FreeListAllocator>> = Mutex::new(None);
}

/// Initialize the kernel heap.
///
/// # Safety
///
/// `start` must point to the virtual address of the heap region and `HEAP_SIZE`
/// bytes of physical frames must be available (both in the virtual address
/// space and physical frames).
pub unsafe fn heap_alloc_init(start: usize) {
    debug_assert!(is_mapping_aligned!(
        start,
        paging::MappingLevel::FourKilobyte
    ));

    // Map the heap.
    for i in (0..HEAP_SIZE).step_by(paging::L0_PAGE_SIZE) {
        let frame = mem::frame_alloc().expect("Out of memory");
        paging::map(
            start + i,
            frame as usize,
            paging::pte_flags::READ | paging::pte_flags::WRITE | paging::pte_flags::VALID,
        );
    }

    log::info!(
        "Kernel heap: {:p}..{:p}",
        start as *const u8,
        (start + HEAP_SIZE) as *const u8
    );

    // Initialize the allocator.
    *HEAP_ALLOCATOR.lock() = Some(FreeListAllocator::new(start, HEAP_SIZE));
}

/// Allocate `size` bytes in the kernel heap.
pub fn kmalloc(size: usize) -> Option<*mut u8> {
    match HEAP_ALLOCATOR.lock().as_mut() {
        Some(allocator) => allocator.alloc(size),
        None => None,
    }
}

/// Calculate heap usage; returns `(free, capacity)`.
pub fn heap_space() -> (usize, usize) {
    match HEAP_ALLOCATOR.lock().as_ref() {
        Some(allocator) => (allocator.free_space(), allocator.size),
        None => (0, 0),
    }
}

/// Returns true if the heap has no allocations.
pub fn heap_empty() -> bool {
    match HEAP_ALLOCATOR.lock().as_ref() {
        Some(allocator) => allocator.free_space() == allocator.size - size_of::<u32>(),
        None => false,
    }
}

/// Free a heap allocation.
///
/// # Safety
///
/// `ptr` must be a pointer returned by `kmalloc()`.
pub unsafe fn kfree(ptr: *mut u8) {
    if let Some(allocator) = HEAP_ALLOCATOR.lock().as_mut() {
        allocator.free(ptr)
    }
}

struct GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match kmalloc(layout.size()) {
            Some(ptr) => ptr,
            None => null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        kfree(ptr)
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Failed to allocate: {:?}", layout);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Block {
    size: u32,
    next: *mut Block,
}

unsafe impl Sync for Block {}
unsafe impl Send for Block {}

impl Block {
    /// Create a new block. `size` is the size of the whole block including the
    /// header.
    pub fn new(size: usize) -> Block {
        Block {
            size: size as u32,
            next: null_mut(),
        }
    }

    pub unsafe fn from_ptr(ptr: *mut u8) -> Option<&'static mut Block> {
        (ptr.sub(size_of::<u32>()) as *mut Block).as_mut()
    }

    pub fn next(&mut self) -> Option<&Block> {
        unsafe { self.next.as_ref() }
    }

    pub fn next_mut(&mut self) -> Option<&mut Block> {
        unsafe { self.next.as_mut() }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Block> {
        successors(Some(self), move |b| unsafe { b.next.as_ref() })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Block> {
        successors(Some(self), move |b| unsafe { b.next.as_mut() })
    }

    pub fn allocation(&mut self) -> *mut u8 {
        unsafe { (addr_of!(*self) as *mut u8).add(size_of::<u32>()) }
    }

    pub fn space(&self) -> usize {
        self.size as usize - size_of::<u32>()
    }

    pub fn boundary(&self) -> *const u8 {
        unsafe { (addr_of!(*self) as *const u8).add(self.size as usize) }
    }

    pub unsafe fn link(&mut self, prev: &mut Block) {
        if prev.allocation().add(prev.space()) as usize == addr_of!(*self) as usize {
            prev.size += self.size;
            while !prev.next.is_null()
                && prev.allocation().add(prev.space()) as usize == prev.next as usize
            {
                prev.size += prev.next().unwrap().size;
                prev.next = prev.next().unwrap().next;
            }
        } else {
            self.next = prev.next;
            prev.next = addr_of_mut!(*self);
        }
    }

    // Claim memory for a new block with the requested space.
    pub fn split(&mut self, space: usize) -> Option<(*mut u8, *mut Block)> {
        let size = space + size_of::<u32>();
        if size > self.space() {
            None
        } else {
            let allocation = self.allocation();
            let new_block = unsafe { self.allocation().add(space) as *mut Block };

            unsafe {
                (*new_block).next = self.next;
                (*new_block).size = self.size - size as u32;
            }

            self.size = size as u32;

            Some((allocation, new_block))
        }
    }
}

pub struct FreeListAllocator {
    head: Option<&'static mut Block>,
    start: usize,
    size: usize,
    current_allocs: usize,
    total_allocs: usize,
    total_frees: usize,
}

impl fmt::Debug for FreeListAllocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "head: {:p}, free blocks: {}, allocated blocks: {}",
            match &self.head {
                Some(r) => addr_of!(**r),
                None => null_mut(),
            },
            match &self.head {
                Some(r) => r.iter().count(),
                None => 0,
            },
            self.current_allocs,
        )?;
        Ok(())
    }
}

impl FreeListAllocator {
    /// Initialize a linked-list allocator.
    ///
    /// # Safety
    ///
    /// `start` must be a contiguous region of `size` bytes.
    pub unsafe fn new(start: usize, size: usize) -> FreeListAllocator {
        let head = start as *mut Block;
        *head = Block::new(size);

        FreeListAllocator {
            head: (head as *mut Block).as_mut(),
            start,
            size,
            current_allocs: 0,
            total_allocs: 0,
            total_frees: 0,
        }
    }

    pub fn free_space(&self) -> usize {
        match &self.head {
            Some(head) => head.iter().map(|b| b.space()).sum(),
            None => 0,
        }
    }

    pub fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        let size = if size < size_of::<Block>() {
            size_of::<Block>()
        } else {
            size
        };

        match &mut self.head {
            Some(head) => {
                let block = match head.iter_mut().find(|b| b.space() >= size) {
                    Some(b) => b,
                    None => return None,
                };

                let alloc = block.split(size);

                if let Some((alloc, block)) = alloc {
                    unsafe {
                        if head.allocation() == Block::from_ptr(alloc).unwrap().allocation() {
                            self.head = block.as_mut();
                        }
                    }

                    self.current_allocs += 1;
                    self.total_allocs += 1;

                    Some(alloc)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Free an allocation
    ///
    /// # Safety
    ///
    /// `ptr` must be an allocation returned by `self.alloc()`.
    pub unsafe fn free(&mut self, ptr: *mut u8) {
        self.current_allocs -= 1;
        let free_block = Block::from_ptr(ptr).unwrap();
        free_block.next = null_mut();

        let prev = self
            .head
            .as_mut()
            .map(|h| {
                h.iter_mut()
                    .find(|b| b.next as usize > addr_of!(*free_block) as usize)
            })
            .unwrap_or(None);

        if let Some(prev) = prev {
            free_block.link(prev);
        } else {
            if let Some(head) = &mut self.head {
                head.link(free_block);
            }
            self.head = Some(free_block);
        }
    }
}

#[cfg(test)]
mod test {}
