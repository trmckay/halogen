use super::{Allocator, AllocatorError};
use crate::{mem::AddressRange, prelude::*};

#[repr(C)]
#[derive(Copy, Clone)]
union NextOrChecksum {
    next: *mut BlockHeader,
    checksum: usize,
}

#[repr(C)]
pub struct BlockHeader {
    size: usize,
    next_or_checksum: NextOrChecksum,
}

const HEADER_SIZE: usize = size_of::<BlockHeader>();
const MAGIC: usize = 0x123456789abcdef;
const MIN_BLOCK_SIZE: usize = 64;

impl BlockHeader {
    /// Claim `size` bytes from the block; return a pointer to the allocation
    /// and a pointer to the next block in the free list (new block or old next)
    fn split(&mut self, size: usize) -> Option<(*mut u8, *mut BlockHeader)> {
        // Large enough to split
        if self.size >= size + HEADER_SIZE + MIN_BLOCK_SIZE {
            let alloc: *mut u8 = self.allocation();
            let new_block = unsafe { alloc.add(size) as *mut BlockHeader };

            unsafe {
                (*new_block).size = self.size - size - HEADER_SIZE;
                (*new_block).next_or_checksum = self.next_or_checksum;
            }

            self.mark_used(size);
            Some((alloc, new_block))
        }
        // Large enough to use completely
        else if self.size >= size {
            let next = unsafe { self.next_or_checksum.next };
            self.mark_used(size);
            Some((self.allocation(), next))
        }
        // Not large enough
        else {
            None
        }
    }

    /// Sign the header of the block with a checksum
    fn mark_used(&mut self, size: usize) {
        debug_assert!(!self.is_likely_used());
        self.size = size;
        self.next_or_checksum.checksum = MAGIC ^ (ptr::addr_of!(*self) as usize);
    }

    /// Check that the checksum field matches the expected value
    fn is_likely_used(&self) -> bool {
        unsafe { self.next_or_checksum.checksum == MAGIC ^ (ptr::addr_of!(*self) as usize) }
    }

    /// Get the next block pointed to by this block, if any
    fn next(&mut self) -> Option<&mut BlockHeader> {
        debug_assert!(!self.is_likely_used());
        unsafe { self.next_or_checksum.next.as_mut() }
    }

    fn allocation<T>(&mut self) -> *mut T {
        (ptr::addr_of_mut!(*self) as usize + HEADER_SIZE) as *mut T
    }
}

unsafe impl Sync for BlockHeader {}
unsafe impl Send for BlockHeader {}

pub struct FreeListAllocator {
    head: *mut BlockHeader,
    range: AddressRange,
}

unsafe impl Sync for FreeListAllocator {}
unsafe impl Send for FreeListAllocator {}

impl FreeListAllocator {
    /// Initialize a free-list allocator
    ///
    /// # Safety
    ///
    /// * `start..start + size` must be exlusively managed by this object
    pub unsafe fn new<T>(start: *mut T, size: usize) -> FreeListAllocator {
        debug_assert!(size > size_of::<BlockHeader>());
        debug_assert_ne!(ptr::null_mut(), start);

        let head = start as *mut BlockHeader;
        (*head).next_or_checksum.next = ptr::null_mut();
        (*head).size = size - HEADER_SIZE;

        FreeListAllocator {
            head,
            range: AddressRange::new(start as usize, size),
        }
    }

    /// Find a block of `size` bytes; return the block's predecessor and the
    /// block itself
    fn find_with_pred(
        &mut self,
        size: usize,
    ) -> Option<(Option<&mut BlockHeader>, &mut BlockHeader)> {
        let mut prev = None;

        for block in self.into_iter() {
            let block = unsafe { block.as_mut()? };
            if (*block).size >= size {
                return Some((prev, block));
            }
            prev = Some(block);
        }

        None
    }
}

impl IntoIterator for &mut FreeListAllocator {
    type IntoIter = FreeListIterator;
    type Item = *mut BlockHeader;

    fn into_iter(self) -> Self::IntoIter {
        FreeListIterator { curr: self.head }
    }
}

impl Allocator for FreeListAllocator {
    fn alloc_bytes(&mut self, count: usize) -> Result<*mut u8, AllocatorError> {
        match self.find_with_pred(count) {
            Some((pred, block)) => {
                match block.split(count) {
                    // Block was split
                    Some((alloc, next)) => {
                        // Point the predecessor or head to the new block
                        match pred {
                            Some(pred) => pred.next_or_checksum.next = next,
                            None => self.head = next,
                        };

                        Ok(alloc)
                    }
                    // This should never happen
                    None => panic!("block selected as large enough, then failed to get space"),
                }
            }
            None => Err(AllocatorError::OutOfSpace(count)),
        }
    }

    unsafe fn free<T>(&mut self, ptr: *mut T) -> Result<(), AllocatorError> {
        let curr = match (ptr as *mut BlockHeader).sub(1).as_mut() {
            None => return Err(AllocatorError::InvalidFree(ptr as usize)),
            Some(block) => block,
        };

        if !curr.is_likely_used() {
            return Err(AllocatorError::InvalidFree(ptr as usize));
        }

        // Search for the predecessor of the allocation to be freed
        match self.into_iter().find(|&block| {
            match (*block).next() {
                // Predecessor if the block lies before the allocation and its next lies after
                // the allocation
                Some(next) => {
                    (ptr::addr_of!(*next) as usize) > (ptr as usize)
                        && (ptr::addr_of!(*block) as usize) < (ptr as usize)
                }
                // Predecessor if the block lies before the allocation and it has no next
                None => (ptr::addr_of!(*block) as usize) < (ptr as usize),
            }
        }) {
            // There is a predecessor block; adjust the list
            Some(prev) => {
                // New block points to old next; previous points to new block
                let next = (*prev).next_or_checksum;
                (*prev).next_or_checksum.next = ptr::addr_of_mut!(*curr);
                curr.next_or_checksum = next;
            }
            // There is no predecessor block; this block becomes the head
            None => {
                curr.next_or_checksum.next = self.head;
                self.head = ptr::addr_of_mut!(*curr);
            }
        };

        Ok(())
    }
}

/// Iterator over all the free blocks in a `FreeListAllocator`
pub struct FreeListIterator {
    curr: *mut BlockHeader,
}

impl Iterator for FreeListIterator {
    type Item = *mut BlockHeader;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr;
        unsafe {
            if !curr.is_null() {
                self.curr = (*self.curr).next_or_checksum.next;
                Some(curr)
            } else {
                None
            }
        }
    }
}
