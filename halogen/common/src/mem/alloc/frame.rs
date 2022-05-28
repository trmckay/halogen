//! This module provides an allocator that can issue and free physical frames of
//! arbitrary size. If frames are never freed, it can function as a bump
//! allocator.

use crate::{
    align_down,
    mem::{Address, PhysicalAddress, VirtualAddress},
};

#[derive(Debug, Clone)]
#[repr(C, align(4096))]
struct FreeFrame(*mut FreeFrame);

/// Physical frame allocator
///
/// Generic with respect to the size of a frame/block
pub struct FrameAllocator<'a, const B: usize> {
    head: Option<*mut FreeFrame>,
    arena: Option<&'a mut [[u8; B]]>,
    virt_offset: isize,
    brk: usize,
}

unsafe impl<'a, const B: usize> Sync for FrameAllocator<'a, B> {}
unsafe impl<'a, const B: usize> Send for FrameAllocator<'a, B> {}

/// Physical frame allocator
impl<'a, const B: usize> FrameAllocator<'a, B> {
    pub const fn new_uninit() -> FrameAllocator<'a, B> {
        FrameAllocator {
            head: None,
            arena: None,
            virt_offset: 0,
            brk: 0,
        }
    }

    /// Create a frame allocator for a specific arena.
    ///
    /// - `arena` is a linear mapping of the physical memory to be managed, i.e.
    ///   virtual addresses.
    /// - `virt_offset` is `arena`'s offset from the physical base.
    ///
    /// # Safety
    ///
    /// - Not idempotent.
    pub unsafe fn init(&mut self, arena: &'a mut [[u8; B]], virt_offset: isize) {
        #[cfg(not(test))]
        debug_assert!(arena.as_ptr().is_aligned_to(B));

        self.head = None;
        self.virt_offset = virt_offset;
        self.arena = Some(arena);
        self.brk = 0;
    }

    /// Create a frame allocator for a specific arena
    ///
    /// # Safety
    ///
    /// - The memory region must be exclusively managed by this structure.
    pub unsafe fn new(arena: &'a mut [[u8; B]], virt_offset: isize) -> FrameAllocator<'a, B> {
        #[cfg(not(test))]
        debug_assert!(arena.as_ptr().is_aligned_to(B));

        FrameAllocator {
            head: None,
            brk: 0,
            virt_offset,
            arena: Some(arena),
        }
    }

    pub fn size(&self) -> usize {
        match &self.arena {
            Some(a) => a.len() * B,
            None => 0,
        }
    }

    pub fn virt_offset(&self) -> isize {
        self.virt_offset
    }

    /// Return the count of frames before and the first byte after which this
    /// allocator has no issued frames.
    pub fn boundary(&self) -> Option<(usize, PhysicalAddress)> {
        Some((
            self.brk,
            PhysicalAddress(self.arena.as_ref()?.get(self.brk)?.as_ptr() as usize),
        ))
    }

    /// Return true if the allocator contains a physical address.
    pub fn contains(&self, addr: PhysicalAddress) -> bool {
        match &self.arena {
            Some(a) => {
                a.as_ptr_range()
                    .contains(&(self.phys_to_virt(addr).as_ptr() as *const [u8; B]))
            }
            None => false,
        }
    }

    #[inline]
    fn virt_to_phys(&self, addr: VirtualAddress) -> PhysicalAddress {
        addr.add_offset(-self.virt_offset).as_phys()
    }

    #[inline]
    fn phys_to_virt(&self, addr: PhysicalAddress) -> VirtualAddress {
        addr.add_offset(self.virt_offset).as_virt()
    }

    /// Allocate a new physical frame.
    pub fn alloc(&mut self) -> Option<PhysicalAddress> {
        unsafe {
            let arena = self.arena.as_mut()?;
            match self.head {
                // No frames have been released and not re-issued
                None => {
                    let frame = VirtualAddress::from_ptr(arena.get_mut(self.brk)?.as_ptr());
                    self.brk += 1;
                    Some(self.virt_to_phys(frame))
                }
                // Some frames are waiting to be re-issued
                Some(head) => {
                    let frame = VirtualAddress::from_ptr(head);
                    let next = (*head).0;

                    self.head = if next.is_null() { None } else { Some(next) };
                    Some(self.virt_to_phys(frame))
                }
            }
        }
    }

    /// Free a frame.
    ///
    /// # Safety
    ///
    /// - Must be called with a valid physical frame.
    pub unsafe fn free(&mut self, frame: PhysicalAddress) {
        assert!(self.contains(frame));
        let frame: usize = align_down!(usize::from(frame), B);
        let new_head = frame.as_mut_ptr() as *mut FreeFrame;

        if let Some(old_head) = self.head {
            (*new_head).0 = old_head;
        } else {
            (*new_head).0 = core::ptr::null_mut();
        }

        self.head = Some(new_head);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bump() {
        let mut buf = vec![[0; 4096]; 16];
        let start = buf.as_ptr() as usize;
        let mut allocator: FrameAllocator<4096> =
            unsafe { FrameAllocator::new(buf.as_mut_slice(), 0) };

        for offset in 0..16 {
            assert_eq!(start + (offset * 4096), allocator.alloc().unwrap().into());
        }

        assert!(allocator.alloc().is_none())
    }

    #[test]
    fn virt_offset_positive() {
        let mut buf = vec![[0; 4096]; 16];
        let start = buf.as_ptr() as usize;

        // Arena is mapped 128 bytes above the actual physical frames
        let mut allocator: FrameAllocator<4096> =
            unsafe { FrameAllocator::new(buf.as_mut_slice(), 128) };

        // So, the addresses returned should be -128
        for offset in 0..16 {
            assert_eq!(
                start + (offset * 4096),
                allocator.alloc().unwrap().add_offset(128).into()
            );
        }

        assert!(allocator.alloc().is_none())
    }

    #[test]
    fn virt_offset_negative() {
        let mut buf = vec![[0; 4096]; 16];
        let start = buf.as_ptr() as usize;

        // Arena is mapped 128 bytes above the actual physical frames
        let mut allocator: FrameAllocator<4096> =
            unsafe { FrameAllocator::new(buf.as_mut_slice(), -128) };

        // So, the addresses returned should be -128
        for offset in 0..16 {
            assert_eq!(
                start + (offset * 4096),
                allocator.alloc().unwrap().add_offset(-128).into()
            );
        }

        assert!(allocator.alloc().is_none())
    }

    #[test]
    fn re_issue_1() {
        let mut buf = vec![[0; 4096]; 16];
        let start = buf.as_ptr() as usize;
        let mut allocator: FrameAllocator<4096> =
            unsafe { FrameAllocator::new(buf.as_mut_slice(), 0) };

        let mut last_frame = PhysicalAddress(0);
        for offset in 0..16 {
            last_frame = allocator.alloc().unwrap();
            assert_eq!(start + (offset * 4096), last_frame.into());
        }

        assert!(allocator.alloc().is_none());

        unsafe {
            allocator.free(last_frame);
        }

        assert_eq!(
            usize::from(last_frame),
            usize::from(allocator.alloc().unwrap())
        );
    }

    #[test]
    fn re_issue_2() {
        let mut buf = vec![[0; 4096]; 16];
        let mut allocator: FrameAllocator<4096> =
            unsafe { FrameAllocator::new(buf.as_mut_slice(), 0) };

        let frames = (0..16)
            .map(|_| allocator.alloc().unwrap())
            .collect::<Vec<PhysicalAddress>>();

        unsafe {
            allocator.free(frames[3]);
            allocator.free(frames[7]);
        }

        assert_eq!(
            usize::from(frames[7]),
            usize::from(allocator.alloc().unwrap())
        );

        assert_eq!(
            usize::from(frames[3]),
            usize::from(allocator.alloc().unwrap())
        );
    }
}
