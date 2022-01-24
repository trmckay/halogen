use crate::mem::{KERNEL_SIZE, KERNEL_START_PHYS, KERNEL_START_VIRT, L0_PAGE_SIZE, PAGING_EN};

pub const PAGE_ALLOC_BLOCK_SIZE: usize = L0_PAGE_SIZE;
pub const PAGE_ALLOC_NUM_BLOCKS: usize = (256) * 8; // = 8M
pub const PAGE_ALLOC_SIZE: usize = PAGE_ALLOC_BLOCK_SIZE * PAGE_ALLOC_NUM_BLOCKS;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum BlockStatus {
    Free,
    Used,
    Boundary,
}

/// A physical page bitmap allocator
#[repr(C, packed)]
pub struct BitmapPageAllocator {
    map: [BlockStatus; PAGE_ALLOC_NUM_BLOCKS],
    arena: usize,
}

impl BitmapPageAllocator {
    pub fn new(arena: usize) -> BitmapPageAllocator {
        BitmapPageAllocator {
            map: [BlockStatus::Free; PAGE_ALLOC_NUM_BLOCKS],
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
        (self.arena + (PAGE_ALLOC_BLOCK_SIZE * block_num)) as *mut u8
    }

    pub fn alloc(&mut self, n: usize) -> Option<*mut u8> {
        let mut alloc_start = 0;
        let mut found = 0;

        for i in 0..PAGE_ALLOC_NUM_BLOCKS {
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
        let mut pos = ((ptr as usize) - self.arena) / PAGE_ALLOC_BLOCK_SIZE;
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

/// Get the kernel heap bitmap from its reserved spot
pub fn get_bitmap() -> &'static mut BitmapPageAllocator {
    unsafe {
        let ptr = (match PAGING_EN {
            0 => KERNEL_START_PHYS,
            _ => KERNEL_START_VIRT,
        } + KERNEL_SIZE) as *mut BitmapPageAllocator;
        ptr.as_mut().expect("Kernel heap is null")
    }
}

/// Alllocate physical pages from the kernel heap
pub fn palloc(pages: usize) -> Option<*mut u8> {
    let map = get_bitmap();
    map.alloc(pages)
}

/// Free an allocations pages
pub fn pfree(ptr: *mut u8) {
    let map = get_bitmap();
    map.free(ptr);
}
