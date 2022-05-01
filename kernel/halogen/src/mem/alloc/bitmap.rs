use super::{Allocator, AllocatorError};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum BlockStatus {
    Free,
    Used,
    Boundary,
}

/// A physical page bitmap allocator
///
/// TODO: It would be nice if the block size and count were provided to the
/// contructor instead of as generics.
///
/// # Generics
///
/// * `N`: number of blocks
/// * `S`: size of each block
#[repr(C, packed)]
pub struct StaticBitmap<const N: usize, const S: usize> {
    map: [BlockStatus; N],
    arena: *mut u8,
}

unsafe impl<const N: usize, const S: usize> Sync for StaticBitmap<N, S> {}
unsafe impl<const N: usize, const S: usize> Send for StaticBitmap<N, S> {}

impl<const N: usize, const S: usize> StaticBitmap<N, S> {
    /// # Safety
    ///
    /// * `arena` must be a pointer to `N * S` free bytes
    pub unsafe fn new(arena: *mut u8) -> StaticBitmap<N, S> {
        StaticBitmap {
            map: [BlockStatus::Free; N],
            arena,
        }
    }

    fn claim(&mut self, block_num: usize, size: usize) {
        for i in 0..(size - 1) {
            self.map[block_num + i] = BlockStatus::Used;
        }
        self.map[block_num + size - 1] = BlockStatus::Boundary;
    }

    fn to_ptr<T>(&self, block_num: usize) -> *mut T {
        unsafe { (self.arena.add(S * block_num)) as *mut T }
    }

    pub fn boundary(&self) -> *const u8 {
        let mut last = 0;
        for i in 0..N {
            if let BlockStatus::Used | BlockStatus::Boundary = self.map[i] {
                last = i;
            }
        }
        unsafe { self.arena.add(S * (last + 1)) }
    }
}

impl<const N: usize, const S: usize> Allocator for StaticBitmap<N, S> {
    fn alloc_bytes(&mut self, size: usize) -> Result<*mut u8, AllocatorError> {
        let block_count = size / S + 1;
        let mut alloc_start = 0;
        let mut found = 0;

        for i in 0..N {
            match self.map[i] {
                BlockStatus::Free => {
                    if found == 0 {
                        alloc_start = i;
                    }
                    found += 1;
                    if found == block_count {
                        self.claim(alloc_start, found);
                        return Ok(self.to_ptr(alloc_start));
                    }
                }
                _ => {
                    found = 0;
                }
            }
        }

        Err(AllocatorError::OutOfSpace(size))
    }

    unsafe fn free<T>(&mut self, ptr: *mut T) -> Result<(), AllocatorError> {
        let mut pos = ((ptr as usize) - (self.arena as usize)) / (N * S);
        loop {
            debug_assert_ne!(BlockStatus::Free, self.map[pos]);
            match self.map[pos] {
                BlockStatus::Used => {
                    self.map[pos] = BlockStatus::Free;
                    pos += 1;
                }
                BlockStatus::Boundary => {
                    self.map[pos] = BlockStatus::Free;
                    return Ok(());
                }
                _ => return Ok(()),
            };
        }
    }
}
