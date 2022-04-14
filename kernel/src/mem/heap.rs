extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use core::{
    iter::successors,
    mem::size_of,
    ptr::{addr_of, addr_of_mut, null_mut},
};

use crate::{is_mapping_aligned, mem, mem::paging, prelude::*};

pub const HEAP_SIZE: usize = 16 * 1024 * 1024;

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

    // Initialize the allocator.
    *HEAP_ALLOCATOR.lock() = Some(FreeListAllocator::new(start, HEAP_SIZE));
}

/// Allocate `size` bytes in the kernel heap
pub fn kmalloc(size: usize) -> Option<*mut u8> {
    match HEAP_ALLOCATOR.lock().as_mut() {
        Some(allocator) => {
            let ptr = allocator.alloc(size);
            match ptr {
                Some(ptr) => trace!("kmalloc({}) -> {:p}", size, ptr),
                None => error!("kmalloc({}): allocator returned None", size),
            }
            ptr
        }
        None => {
            error!("kmalloc({}): could not acquire lock on allocator", size);
            None
        }
    }
}

/// Calculate heap usage
pub fn free_space() -> usize {
    match HEAP_ALLOCATOR.lock().as_ref() {
        Some(allocator) => allocator.free_space(),
        None => 0,
    }
}

/// Calculate number of heap blocks free
pub fn free_blocks() -> usize {
    match HEAP_ALLOCATOR.lock().as_ref() {
        Some(allocator) => {
            allocator
                .head
                .as_ref()
                .map(|h| h.iter().count())
                .unwrap_or(0)
        }
        None => 0,
    }
}

/// Calculate number of heap blocks in use
pub fn used_blocks() -> usize {
    match HEAP_ALLOCATOR.lock().as_ref() {
        Some(allocator) => allocator.current_allocs,
        None => 0,
    }
}

/// Free a heap allocation
///
/// # Safety
///
/// `ptr` must be a pointer returned by `kmalloc()`
pub unsafe fn kfree(ptr: *mut u8) {
    trace!("kfree({:p})", ptr);

    match HEAP_ALLOCATOR.lock().as_mut() {
        Some(allocator) => {
            allocator.free(ptr);
        }
        None => error!("kfree({:p}): could not acquire lock on allocator", ptr),
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
    panic!("Global allocator error: size={}", layout.size());
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Block {
    size: u32,
    next: *mut Block,
    prev: *mut Block,
}

unsafe impl Sync for Block {}
unsafe impl Send for Block {}

impl Block {
    /// Create a new block
    ///
    /// `size` is the size of the whole block including the header
    fn new(size: usize) -> Block {
        Block {
            size: size as u32,
            next: null_mut(),
            prev: null_mut(),
        }
    }

    /// Cast a raw pointer to a `Block` reference
    unsafe fn from_ptr(ptr: *mut u8) -> Option<&'static mut Block> {
        (ptr.sub(size_of::<u32>()) as *mut Block).as_mut()
    }

    /// Get a reference to this block's next
    fn next(&mut self) -> Option<&Block> {
        unsafe { self.next.as_ref() }
    }

    /// Get a mutable reference to this block's next
    fn next_mut(&mut self) -> Option<&mut Block> {
        unsafe { self.next.as_mut() }
    }

    /// Get a reference to this block's next
    fn prev(&mut self) -> Option<&Block> {
        unsafe { self.prev.as_ref() }
    }

    /// Get a mutable reference to this block's next
    fn prev_mut(&mut self) -> Option<&mut Block> {
        unsafe { self.prev.as_mut() }
    }

    /// Iterate over the linked list starting at this block
    fn iter(&self) -> impl Iterator<Item = &Block> {
        successors(Some(self), move |b| unsafe { b.next.as_ref() })
    }

    /// Iterate mutably over the linked list starting at this block
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Block> {
        successors(Some(self), move |b| unsafe { b.next.as_mut() })
    }

    /// Get a pointer to the usable allocation of this block
    fn allocation(&mut self) -> *mut u8 {
        unsafe { (addr_of!(*self) as *mut u8).add(size_of::<u32>()) }
    }

    /// Get the count of usable bytes in this block
    fn space(&self) -> usize {
        self.size as usize - size_of::<u32>()
    }

    /// Get a pointer to the end of this block
    fn boundary(&self) -> *const u8 {
        unsafe { (addr_of!(*self) as *const u8).add(self.size as usize) }
    }

    /// Link another block to this block
    unsafe fn link(&mut self, prev: &mut Block) {
        if prev.boundary() as usize == addr_of!(*self) as usize {
            prev.size += self.size;
            self.prev = addr_of_mut!(*prev);
            while !prev.next.is_null()
                && prev.allocation().add(prev.space()) as usize == prev.next as usize
            {
                prev.size += prev.next().unwrap().size;
                prev.next = prev.next().unwrap().next;
            }
        } else {
            self.next = prev.next;
            self.prev = addr_of_mut!(*prev);
            prev.next = addr_of_mut!(*self);
        }
    }

    /// Reserve `size` bytes for usage in the current block and create a new
    /// block at the end of the allocation
    fn reserve(&mut self, size: usize) -> Result<Option<*mut Block>, ()> {
        let used_block_size = (size + size_of::<u32>()) as u32;
        // A new block can fit in the free space
        if (used_block_size as usize) < self.space() {
            let new_block = unsafe { self.allocation().add(size) as *mut Block };

            unsafe {
                // Current block is now in use, previous' next must point to the new block
                if !self.prev.is_null() {
                    (*self.prev).next = new_block;
                }

                (*new_block).next = self.next;
                (*new_block).prev = self.prev;
                (*new_block).size = self.size - used_block_size;
            }

            self.size = used_block_size;

            Ok(Some(new_block))
        }
        // The whole block can be used for the allocation
        else if size <= self.space() {
            Ok(None)
        }
        // There is not enough space in this block
        else {
            Err(())
        }
    }
}

struct FreeListAllocator {
    head: Option<&'static mut Block>,
    start: usize,
    size: usize,
    current_allocs: usize,
    total_allocs: usize,
    total_frees: usize,
}

impl FreeListAllocator {
    /// Initialize a linked-list allocator
    ///
    /// # Safety
    ///
    /// `start` must be a contiguous region of `size` bytes
    unsafe fn new(start: usize, size: usize) -> FreeListAllocator {
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

    /// Calculate the amount of usable space in the arena
    fn free_space(&self) -> usize {
        match &self.head {
            Some(head) => head.iter().map(|b| b.space()).sum(),
            None => 0,
        }
    }

    /// Allocate some bytes
    fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        let size = core::cmp::max(size, size_of::<Block>());
        let (used_block, new_block) = self
            .head
            .as_mut()
            .and_then(|head| head.iter_mut().find(|b| b.space() >= size))
            .map(|block| {
                let new_block = block.reserve(size).expect("couldn't reserve from block");
                (block, new_block)
            })?;

        let ptr = used_block.allocation();

        // Replace the head if necessary
        if ptr::eq(used_block, *self.head.as_ref().unwrap()) {
            unsafe { self.head = new_block.and_then(|b| b.as_mut()) }
        }

        self.total_allocs += 1;
        self.current_allocs += 1;

        Some(ptr)
    }

    /// Free an allocation
    ///
    /// # Safety
    ///
    /// `ptr` must be an allocation returned by `self.alloc()`
    unsafe fn free(&mut self, ptr: *mut u8) {
        let free_block = Block::from_ptr(ptr).unwrap();

        free_block.next = null_mut();
        free_block.prev = null_mut();

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

        self.current_allocs -= 1;
    }
}
