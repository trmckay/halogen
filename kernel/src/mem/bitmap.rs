#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum BlockStatus {
    Free,
    Used,
    Boundary,
}

/// A physical page bitmap allocator
///
/// Generics:
/// * `N`: number of blocks
/// * `S`: size of each block
#[repr(C, packed)]
pub struct Bitmap<const N: usize, const S: usize> {
    map: [BlockStatus; N],
    arena: *mut u8,
}

impl<const N: usize, const S: usize> Bitmap<N, S> {
    pub fn new(arena: *mut u8) -> Bitmap<N, S> {
        Bitmap {
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

    fn to_ptr(&self, block_num: usize) -> *mut u8 {
        unsafe { (self.arena.add(S * block_num)) as *mut u8 }
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

    pub fn alloc(&mut self, n: usize) -> Option<*mut u8> {
        let mut alloc_start = 0;
        let mut found = 0;

        for i in 0..N {
            match self.map[i] {
                BlockStatus::Free => {
                    if found == 0 {
                        alloc_start = i;
                    }
                    found += 1;
                    if found == n {
                        self.claim(alloc_start, found);
                        return Some(self.to_ptr(alloc_start));
                    }
                }
                _ => {
                    found = 0;
                }
            }
        }

        None
    }

    pub fn free(&mut self, ptr: *const u8) {
        let mut pos = ((ptr as usize) - (self.arena as usize)) / N;
        loop {
            match self.map[pos] {
                BlockStatus::Used => {
                    self.map[pos] = BlockStatus::Free;
                    pos += 1;
                }
                BlockStatus::Boundary => {
                    self.map[pos] = BlockStatus::Free;
                    return;
                }
                _ => unreachable!(),
            };
        }
    }
}
