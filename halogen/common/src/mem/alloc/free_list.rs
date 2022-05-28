//! This module provides a linked-list based implementation of the Rust
//! `GlobalAllocator` interface, i.e. an allocator that has control over some
//! memory and can allocate chunks given a `Layout`.
//!
//! This is not the most elegent implementation of a linked-list. It must be
//! must be `std`/`alloc`-free, so can't use any containers that do dynamic
//! borrow checking. In other words, there's a lot of 'unsafe' to get around
//! interior mutability using raw pointers.
//!
//! With the requirement of only interfacing using `alloc` and `dealloc`, this
//! can still be done safely without reference-counting or cells

use core::{alloc::Layout, fmt::Debug, slice::from_raw_parts_mut};

use super::AllocatorStats;

const MAGIC: usize = 0x1234567890abcdef;

/// Stores metadata about an allocation block.
#[derive(Clone, Copy, Debug)]
pub enum BlockHeader {
    Free(usize, *mut BlockHeader, *mut BlockHeader),
    Used(usize, usize, usize),
}

unsafe impl Sync for BlockHeader {}
unsafe impl Send for BlockHeader {}

const HEADER_SIZE: usize = core::mem::size_of::<BlockHeader>();

impl BlockHeader {
    /// Create a new block from a size
    fn new(size: usize) -> BlockHeader {
        BlockHeader::Free(size, core::ptr::null_mut(), core::ptr::null_mut())
    }

    /// Get a pointer to the block.
    #[inline]
    fn as_ptr(&self) -> *const BlockHeader {
        core::ptr::addr_of!(*self)
    }

    /// Get a mutable pointer to the block.
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut BlockHeader {
        core::ptr::addr_of_mut!(*self)
    }

    /// Create a mutable reference to the header whose *base* is at a pointer.
    #[inline]
    fn from_ptr<T>(ptr: *mut T) -> &'static mut BlockHeader {
        unsafe { (ptr as *mut BlockHeader).as_mut().expect("null pointer") }
    }

    /// Create a mutable reference to the header whose allocation is at a
    /// pointer.
    #[inline]
    fn from_caller_ptr<T>(ptr: *mut T) -> &'static mut BlockHeader {
        unsafe {
            (ptr as *mut BlockHeader)
                .sub(1)
                .as_mut()
                .expect("null pointer")
        }
    }

    /// Returns the size of the block.
    #[inline]
    fn size(&self) -> usize {
        match self {
            BlockHeader::Free(size, _, _) | BlockHeader::Used(size, _, _) => *size,
        }
    }

    /// Returns the size of the block.
    #[inline]
    fn set_size(&mut self, size: usize) {
        match self {
            BlockHeader::Free(this_size, _, _) | BlockHeader::Used(this_size, _, _) => {
                *this_size = size
            }
        }
    }

    /// Return a pointer to the first byte outside this block.
    #[inline]
    fn end(&self) -> *const u8 {
        (self.as_ptr() as usize + self.size() + HEADER_SIZE) as *const u8
    }

    /// Get a mutable reference to the next block.
    fn next(self) -> Option<&'static mut BlockHeader> {
        unsafe { self.next_ptr().as_mut() }
    }

    /// Get a mutable reference to the previous block.
    #[inline]
    fn prev(self) -> Option<&'static mut BlockHeader> {
        unsafe { self.prev_ptr().as_mut() }
    }

    /// Get a pointer to the usable region of a block.
    #[inline]
    fn allocation<T>(&mut self) -> *mut T {
        (self.as_ptr() as usize + HEADER_SIZE) as *mut T
    }

    /// Get the pointer to the next block.
    #[inline]
    unsafe fn next_ptr(self) -> *mut BlockHeader {
        match self {
            BlockHeader::Free(_, _, next) => next,
            _ => panic!("cannot get next for non-free block"),
        }
    }

    /// Get the pointer to the previous block.
    #[inline]
    unsafe fn prev_ptr(self) -> *mut BlockHeader {
        match self {
            BlockHeader::Free(_, prev, _) => prev,
            _ => panic!("cannot get previous for non-free block"),
        }
    }

    /// Set the pointer to the next block.
    #[inline]
    unsafe fn set_next(&mut self, next: *mut BlockHeader) {
        match self {
            BlockHeader::Free(_, _, this_next) => *this_next = next,
            _ => panic!("cannot set next for non-free block"),
        }
    }

    /// Set the pointer to the previous block.
    #[inline]
    unsafe fn set_prev(&mut self, prev: *mut BlockHeader) {
        match self {
            BlockHeader::Free(_, this_prev, _) => *this_prev = prev,
            _ => panic!("cannot set next for non-free block"),
        }
    }

    #[inline]
    fn calc_check(&self) -> usize {
        MAGIC ^ ((self.as_ptr() as usize).overflowing_mul(self.size())).0
    }

    /// Set the block metadata to a checksum and record the new size. Call when
    /// issuing a block to a caller.
    #[inline]
    fn make_used(&mut self, size: usize) {
        match self {
            BlockHeader::Free(_, _, _) => {
                self.set_size(size); // Set this for the checksum calculation.
                let check = self.calc_check();
                *self = BlockHeader::Used(size, check, check);
            }
            _ => panic!("cannot mark a non-free block as used"),
        }
    }

    /// Set the block metadata to a checksum and record the new size. Call when
    /// issuing a block to a caller.
    #[inline]
    fn make_free(&mut self) {
        match self {
            BlockHeader::Used(size, _, _) => {
                *self = BlockHeader::Free(*size, core::ptr::null_mut(), core::ptr::null_mut());
            }
            _ => panic!("cannot mark a non-free block as used"),
        }
    }

    /// Check that the checksum is still valid. A value of `false` may
    /// indicate out-of-bounds writes have corrupted the list.
    #[inline]
    fn is_used_and_valid(&self) -> bool {
        match self {
            BlockHeader::Used(_, ck1, ck2) => {
                let check = self.calc_check();
                check == *ck1 && check == *ck2
            }
            _ => false,
        }
    }

    /// Try to consolidate a block with its neighbors and return true if
    /// successful.
    fn try_coalesce(&mut self) -> bool {
        unsafe {
            // There is a next block.
            if let Some(old_next) = self.next() {
                // The blocks are adjacent.
                if self.end() as usize == self.next_ptr() as usize {
                    // Consume the other block and adjust the list.
                    self.set_size(self.size() + old_next.size() + HEADER_SIZE);
                    self.set_next(old_next.next_ptr());
                    if let Some(new_next) = self.next() {
                        new_next.set_prev(self);
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
    }

    /// Returns true if `self` sits before `other` in memory.
    #[inline]
    fn should_precede(&self, other: &BlockHeader) -> bool {
        (self.as_ptr() as usize) < (other.as_ptr() as usize)
    }

    /// Attempt to coalesce two blocks, linking them by pointer if they are not
    /// adjacent.
    ///
    /// Before:
    ///
    /// ```text
    /// ... <--> [self] <--> [self.next] <--> ...
    /// ```
    ///
    /// After:
    ///
    /// ```text
    /// ... <--> [self] <--> [new] <--> [self.next] <--> ...
    /// ```
    ///
    /// # Safety
    ///
    /// - `new` must not be in the list already.
    unsafe fn push(&mut self, new: &mut BlockHeader) {
        new.set_prev(self.as_mut_ptr());

        if let Some(next) = self.next() {
            next.set_prev(new.as_mut_ptr());
            new.set_next(next.as_mut_ptr());
        } else {
            new.set_next(core::ptr::null_mut());
        }
        self.set_next(new.as_mut_ptr());

        loop {
            if !self.try_coalesce() {
                break;
            }
        }
    }
}

#[derive(Debug)]
pub struct FreeListAllocator<'a, const B: usize> {
    head: *mut BlockHeader,
    arena: &'a mut [u8],
    issued_blocks: usize,
}

unsafe impl<'a, const B: usize> Sync for FreeListAllocator<'a, B> {}
unsafe impl<'a, const B: usize> Send for FreeListAllocator<'a, B> {}

impl<'a, const B: usize> FreeListAllocator<'a, B> {
    /// Initialize a free-list allocator with a region of memory to manage.
    pub fn new(arena: &'a mut [u8]) -> FreeListAllocator<B> {
        assert!(arena.len() > HEADER_SIZE + B);

        let head = BlockHeader::from_ptr(arena.as_mut_ptr());
        *head = BlockHeader::new(arena.len() - HEADER_SIZE);

        FreeListAllocator {
            head: arena.as_ptr() as *mut BlockHeader,
            arena,
            issued_blocks: 0,
        }
    }

    /// Get a mutable reference to the head of the linked-list of free blocks.
    #[inline]
    fn head(&self) -> Option<&'static mut BlockHeader> {
        unsafe { self.head.as_mut() }
    }

    pub fn stats(&self) -> AllocatorStats {
        let bytes_total = self.arena.len();
        let blocks_used = self.issued_blocks;
        let bytes_free = self.into_iter().map(|block| block.size()).sum();
        let bytes_used = bytes_total - (blocks_used * HEADER_SIZE) - bytes_free;
        let bytes_overhead = bytes_total - bytes_free - bytes_used;
        let blocks_free = self.into_iter().count();
        let blocks_total = blocks_free + blocks_used;

        AllocatorStats {
            bytes_free,
            bytes_used,
            bytes_total,
            bytes_overhead,
            blocks_free,
            blocks_used,
            blocks_total,
        }
    }

    /// Returns `true` if the allocator contains the pointer.
    pub fn contains(&self, ptr: *const u8, layout: Layout) -> bool {
        self.arena.as_ptr_range().contains(&ptr)
            && self
                .arena
                .as_ptr_range()
                .contains(&((ptr as usize + layout.size() - 1) as *const u8))
    }

    /// Try to allocate `size` bytes from this block. If possible, adjust the
    /// list and return a pointer to the allocation.
    fn alloc_from_block(
        &mut self,
        block: &'static mut BlockHeader,
        size: usize,
    ) -> Option<*mut u8> {
        // Not enough space
        if block.size() < size {
            None
        }
        // Enough space, but not enough left over to make a smaller block.
        else if block.size() - size < HEADER_SIZE + B {
            // Connect the previous block or head to the used block's next block.
            unsafe {
                match block.prev() {
                    Some(prev) => prev.set_next(block.next_ptr()),
                    None => self.head = block.next_ptr(),
                }
            }

            // If there is a next, update its previous to the used block's previous.
            if let Some(next) = block.next() {
                unsafe {
                    next.set_prev(block.prev_ptr());
                }
            }

            // Mark the block as used and return a pointer to the usable region.
            block.make_used(size);
            Some(block.allocation())
        }
        // Split the block (there is space for a new block).
        else {
            unsafe {
                let new_block = block.allocation::<u8>().add(size) as *mut BlockHeader;

                match block.prev() {
                    Some(prev) => prev.set_next(new_block),
                    None => self.head = new_block,
                }

                if let Some(next) = block.next() {
                    next.set_prev(new_block);
                }

                *new_block = BlockHeader::Free(
                    block.size() - HEADER_SIZE - size,
                    block.prev_ptr(),
                    block.next_ptr(),
                );

                block.make_used(size);
                Some(block.allocation())
            }
        }
    }

    /// Allocate a block with the provided layout. Guaranteed to be at least `B`
    /// and `layout.size()` in size. Returns `core::ptr::null_mut()` if no
    /// suitable allocation is found.
    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(B);
        match self
            .into_iter()
            .find(|block| block.size() >= size)
            .and_then(|block| self.alloc_from_block(block, size))
        {
            Some(ptr) => {
                self.issued_blocks += 1;
                ptr
            }
            None => core::ptr::null_mut(),
        }
    }

    /// Place a new block at the head of this, adjust the successors.
    pub fn set_head(&mut self, block: &mut BlockHeader) {
        unsafe {
            // If there is a head, set it's previous to the new head.
            if let Some(head) = self.head() {
                head.set_prev(block.as_mut_ptr());
            }
            // Point the new head to the old head (or lack of head).
            block.set_next(self.head);
            // Finally, set the head.
            self.head = block.as_mut_ptr();
        }
    }

    /// Allocate some memory and zero it.
    pub fn alloc_zeroed(&mut self, layout: Layout) -> *mut u8 {
        let alloc = self.alloc(layout);

        if !alloc.is_null() {
            unsafe {
                from_raw_parts_mut(alloc, layout.size())
                    .iter_mut()
                    .for_each(|byte| *byte = 0);
            }
        }

        alloc
    }

    /// Deallocate a block that was previously allocated from this allocator.
    /// Will usually panic on invalid frees. May panic if the list is corrupted.
    ///
    /// # Safety
    ///
    /// * Only call
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let block = BlockHeader::from_caller_ptr(ptr);

        // Panic if pointer is out-of-bounds.
        assert!(self.contains(ptr, layout));
        // Panic if there is suspected corruption.
        assert!(block.is_used_and_valid());

        self.issued_blocks -= 1;
        block.make_free();

        match self
            .into_iter()
            .filter(|other| other.should_precede(block))
            .last()
        {
            Some(pred) => pred.push(block),
            None => self.set_head(block),
        }
    }

    /// Returns an false if the heap is suspected corrupted
    pub fn integrity_ok(&self) -> bool {
        unsafe {
            if !self.head.is_null() && !(*self.head).prev_ptr().is_null() {
                return false;
            }

            for block in self.into_iter() {
                // The block is within the allocator's memory.
                if !self.contains(
                    block.as_ptr() as *const u8,
                    Layout::from_size_align(HEADER_SIZE + block.size(), 1).unwrap(),
                ) {
                    return false;
                }
                if !block.next_ptr().is_null() {
                    if (block.next_ptr() as usize)
                        < block.as_ptr() as usize + HEADER_SIZE + block.size()
                    {
                        return false;
                    }
                    if block.next().unwrap().prev_ptr() as usize != block.as_ptr() as usize {
                        return false;
                    }
                }
                if !block.prev_ptr().is_null() {
                    if block.prev_ptr() as usize >= block.as_ptr() as usize {
                        return false;
                    }
                    if block.prev().unwrap().next_ptr() as usize != block.as_ptr() as usize {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl<'a, const B: usize> core::fmt::Display for FreeListAllocator<'a, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Statistics: {:?}", self.stats())?;
        for block in self.into_iter() {
            writeln!(f, "{:p}: {}", block, block.size())?;
        }
        Ok(())
    }
}

/// Iterator over all the free blocks in a `FreeListAllocator`
pub struct FreeListIterator {
    curr: *mut BlockHeader,
}

impl Iterator for FreeListIterator {
    type Item = &'static mut BlockHeader;

    fn next(&mut self) -> Option<&'static mut BlockHeader> {
        if self.curr.is_null() {
            None
        } else {
            unsafe {
                let curr = self.curr.as_mut()?;
                let next = curr.next_ptr();

                self.curr = next;
                Some(curr)
            }
        }
    }
}

impl<'a, const B: usize> IntoIterator for &FreeListAllocator<'a, B> {
    type IntoIter = FreeListIterator;
    type Item = &'static mut BlockHeader;

    fn into_iter(self) -> Self::IntoIter {
        FreeListIterator { curr: self.head }
    }
}

#[cfg(test)]
mod test {
    use core::slice::from_raw_parts_mut;

    use super::*;
    use crate::mem::KIB;

    #[test]
    fn simple() {
        unsafe {
            let mut buf = vec![0; 4 * KIB];
            let mut allocator: FreeListAllocator<64> = FreeListAllocator::new(buf.as_mut_slice());

            let free_before = allocator.stats().bytes_free;

            let l1 = Layout::new::<[u8; KIB / 2]>();
            let p1 = allocator.alloc(l1) as *const u8;

            assert!(!p1.is_null());
            assert!(allocator.contains(p1, l1));
            assert!(allocator.integrity_ok());

            let l2 = Layout::new::<[u8; KIB]>();
            let p2 = allocator.alloc(l2) as *const u8;

            assert!(allocator.integrity_ok());
            assert!(!p2.is_null());
            assert!(allocator.contains(p2, l2));
            assert_eq!(p2 as usize, p1 as usize + HEADER_SIZE + l1.size());
            assert_eq!(
                l2.size(),
                BlockHeader::from_caller_ptr(p2 as *mut u8).size()
            );
            assert_eq!(allocator.head as usize, p2.add(l2.size()) as usize);
            assert_ne!(p1, p2);
            assert_eq!(
                free_before - l1.size() - l2.size() - (2 * HEADER_SIZE),
                allocator.stats().bytes_free
            );
        }
    }

    #[test]
    fn too_large() {
        let mut buf = vec![0; 128];
        let mut allocator: FreeListAllocator<64> = FreeListAllocator::new(buf.as_mut_slice());

        assert!(allocator
            .alloc(Layout::from_size_align(128 - HEADER_SIZE + 1, 1).unwrap())
            .is_null());
        assert!(allocator.integrity_ok());
    }

    #[test]
    fn perfect_fit() {
        let mut buf = vec![0; 128];
        let mut allocator: FreeListAllocator<64> = FreeListAllocator::new(buf.as_mut_slice());

        assert!(!allocator
            .alloc(Layout::from_size_align(128 - HEADER_SIZE, 1).unwrap())
            .is_null());
        assert!(allocator.integrity_ok());
    }

    #[test]
    fn out_of_space() {
        let mut buf = vec![0; 128];
        let mut allocator: FreeListAllocator<64> = FreeListAllocator::new(buf.as_mut_slice());

        assert!(!allocator
            .alloc(Layout::from_size_align(128 - HEADER_SIZE, 1).unwrap())
            .is_null());

        assert!(allocator
            .alloc(Layout::from_size_align(1, 1).unwrap())
            .is_null());

        let mut buf = vec![0; 128];
        let mut allocator: FreeListAllocator<64> = FreeListAllocator::new(buf.as_mut_slice());

        assert!(!allocator
            .alloc(Layout::from_size_align(128 / 2, 1).unwrap())
            .is_null());

        assert!(allocator
            .alloc(Layout::from_size_align(128 / 2 + 1, 1).unwrap())
            .is_null());

        assert!(allocator.integrity_ok());
    }

    #[test]
    #[should_panic]
    fn invalid_free() {
        let mut buf = vec![0; 128];
        let mut allocator: FreeListAllocator<64> = FreeListAllocator::new(buf.as_mut_slice());

        let layout = Layout::from_size_align(64, 1).unwrap();
        let ptr = allocator.alloc(layout);

        unsafe {
            allocator.dealloc(ptr.sub(1), layout);
        }
    }

    #[test]
    fn allocate_between() {
        let mut buf = vec![0; 8 * KIB];
        let mut allocator: FreeListAllocator<64> = FreeListAllocator::new(buf.as_mut_slice());
        let arr_size = 1000;

        let layout = Layout::from_size_align(arr_size, 1).unwrap();

        assert!(!allocator.alloc_zeroed(layout).is_null());

        let middle_block = allocator.alloc_zeroed(layout);
        assert!(!middle_block.is_null());

        assert!(!allocator.alloc_zeroed(layout).is_null());

        unsafe {
            allocator.dealloc(middle_block, layout);
        }
        assert_eq!(middle_block as usize - HEADER_SIZE, allocator.head as usize);

        assert!(allocator.integrity_ok());

        let larger_layout = Layout::from_size_align(arr_size + 1, 1).unwrap();
        assert!(!allocator.alloc_zeroed(larger_layout).is_null());

        assert!(allocator.integrity_ok());
    }

    #[test]
    fn stress_1() {
        unsafe {
            let mut buf = vec![0; 8 * KIB];
            let mut allocator: FreeListAllocator<64> = FreeListAllocator::new(buf.as_mut_slice());

            assert!(allocator.integrity_ok());

            let orig_head = allocator.head;
            let free_before = allocator.stats().bytes_free;

            let l1 = Layout::new::<[u8; KIB]>();
            let p1 = allocator.alloc(l1) as *const u8;

            assert!(!p1.is_null());
            assert!(allocator.arena.as_ptr_range().contains(&p1));
            assert!(allocator.integrity_ok());

            let l2 = Layout::new::<[u8; 32]>();
            let p2 = allocator.alloc(l2) as *const u8;

            assert!(!p2.is_null());
            assert!(allocator.arena.as_ptr_range().contains(&p2));
            assert_eq!(p2 as usize, p1 as usize + HEADER_SIZE + l1.size());
            assert_eq!(64, BlockHeader::from_caller_ptr(p2 as *mut u8).size());
            assert_eq!(allocator.head as usize, p2.add(64) as usize);
            assert!(allocator.integrity_ok());

            let free_during = allocator.stats().bytes_free;

            let old_head = allocator.head;
            allocator.dealloc(p1 as *mut u8, l1);

            assert_eq!(allocator.head, (p1 as *mut BlockHeader).sub(1));
            assert_eq!(old_head, allocator.head().unwrap().next_ptr());
            assert!(allocator.head().unwrap().prev().is_none());
            assert_eq!(free_during + l1.size(), allocator.stats().bytes_free);
            assert_eq!(
                l1.size(),
                BlockHeader::from_caller_ptr(p1 as *mut u8).size()
            );
            assert!(allocator.integrity_ok());

            allocator.dealloc(p2 as *mut u8, l2);

            assert_eq!(allocator.head, orig_head);
            assert!(allocator.head().unwrap().next().is_none());
            assert_eq!(free_before, allocator.stats().bytes_free);
            assert!(allocator.integrity_ok());
        }
    }

    #[test]
    fn stress_2() {
        for _ in 0..100 {
            let mut buf = vec![0; 20 * KIB];
            let mut allocator: FreeListAllocator<64> = FreeListAllocator::new(buf.as_mut_slice());

            let trials = 10;

            for _ in 0..trials {
                let sizes = [16, 64, 512];

                let usage_before = allocator.stats().bytes_used;
                let free_before = allocator.stats().bytes_free;

                let ptrs = sizes
                    .iter()
                    .map(|&size| {
                        let layout = Layout::from_size_align(size, 1).unwrap();
                        let ptr = allocator.alloc_zeroed(layout);
                        (layout, ptr)
                    })
                    .collect::<Vec<_>>();

                for (i1, (l1, p1)) in ptrs.iter().enumerate() {
                    for (i2, (l2, p2)) in ptrs.iter().enumerate() {
                        let p1 = *p1 as usize;
                        let p2 = *p2 as usize;

                        assert!(
                            i1 == i2
                                || p1.abs_diff(p2)
                                    >= (if p1 < p2 { l1.size() } else { l2.size() }) + HEADER_SIZE
                        );
                    }

                    let slice = unsafe { from_raw_parts_mut(*p1, l1.size()) };

                    slice.iter_mut().for_each(|byte| {
                        assert_eq!(0, *byte);
                        *byte = 7;
                    });
                }

                ptrs.iter()
                    .for_each(|(layout, alloc)| unsafe { allocator.dealloc(*alloc, *layout) });

                assert_eq!(usage_before, allocator.stats().bytes_used);
                assert_eq!(free_before, allocator.stats().bytes_free);
            }
        }
    }

    #[test]
    fn stress_3() {
        let mut buf = vec![0; 8 * KIB];
        let mut allocator: FreeListAllocator<64> = FreeListAllocator::new(buf.as_mut_slice());
        let arr_size = 1000;

        assert!(allocator.integrity_ok());

        let (l1, arr1) = unsafe {
            let layout = Layout::from_size_align(arr_size, 1).unwrap();
            (
                layout,
                from_raw_parts_mut(allocator.alloc_zeroed(layout), arr_size),
            )
        };

        assert!(allocator.integrity_ok());

        arr1.iter_mut().for_each(|byte| *byte = 1);

        assert!(allocator.integrity_ok());

        let (l2, arr2) = unsafe {
            let layout = Layout::from_size_align(arr_size, 1).unwrap();
            (
                layout,
                from_raw_parts_mut(allocator.alloc_zeroed(layout), arr_size),
            )
        };

        assert!(allocator.integrity_ok());

        arr2.iter_mut().for_each(|byte| *byte = 2);
        arr1.iter().for_each(|byte| assert_eq!(1, *byte));

        assert!(allocator.integrity_ok());

        let (l3, arr3) = unsafe {
            let layout = Layout::from_size_align(arr_size, 1).unwrap();
            (
                layout,
                from_raw_parts_mut(allocator.alloc_zeroed(layout), arr_size),
            )
        };

        assert!(allocator.integrity_ok());

        arr3.iter_mut().for_each(|byte| *byte = 3);

        assert!(allocator.integrity_ok());
        arr1.iter().for_each(|byte| assert_eq!(1, *byte));
        arr2.iter().for_each(|byte| assert_eq!(2, *byte));

        arr1.iter_mut().for_each(|byte| *byte = 1);
        arr2.iter_mut().for_each(|byte| *byte = 2);

        assert!(allocator.integrity_ok());
        arr3.iter().for_each(|byte| assert_eq!(3, *byte));

        unsafe {
            allocator.dealloc(arr2.as_mut_ptr(), l2);
        }

        assert!(allocator.integrity_ok());
        arr1.iter().for_each(|byte| assert_eq!(1, *byte));
        arr3.iter().for_each(|byte| assert_eq!(3, *byte));

        let (l4, arr4) = unsafe {
            let layout = Layout::from_size_align(arr_size + 1, 1).unwrap();
            (
                layout,
                from_raw_parts_mut(allocator.alloc_zeroed(layout), arr_size + 1),
            )
        };

        assert!(allocator.integrity_ok());
        assert!(
            arr4.as_ptr() > arr3.as_ptr()
                && arr4.as_ptr() > arr2.as_ptr()
                && arr4.as_ptr() > arr1.as_ptr()
        );
        arr1.iter().for_each(|byte| assert_eq!(1, *byte));
        arr3.iter().for_each(|byte| assert_eq!(3, *byte));

        arr4.iter_mut().for_each(|byte| *byte = 4);

        assert!(allocator.integrity_ok());
        arr1.iter().for_each(|byte| assert_eq!(1, *byte));
        arr3.iter().for_each(|byte| assert_eq!(3, *byte));

        unsafe {
            allocator.dealloc(arr4.as_mut_ptr(), l4);
        }

        println!("{}", allocator);

        assert!(allocator.integrity_ok());
        arr1.iter().for_each(|byte| assert_eq!(1, *byte));
        arr3.iter().for_each(|byte| assert_eq!(3, *byte));

        unsafe {
            allocator.dealloc(arr1.as_mut_ptr(), l1);
        }

        assert!(allocator.integrity_ok());
        arr3.iter().for_each(|byte| assert_eq!(3, *byte));

        unsafe {
            allocator.dealloc(arr3.as_mut_ptr(), l3);
        }

        assert!(allocator.integrity_ok());
    }
}
