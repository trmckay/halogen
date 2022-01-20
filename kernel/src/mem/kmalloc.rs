use crate::{
    mem::{KERNEL_END, L0_PAGE_SIZE},
    size_of,
};

/// Kernel heap is 2M
pub const KHEAP_BLOCK_SIZE: usize = L0_PAGE_SIZE;
pub const KHEAP_NUM_BLOCKS: usize = 64 * 8;
pub const KHEAP_SIZE: usize = KHEAP_BLOCK_SIZE * KHEAP_NUM_BLOCKS;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum BlockStatus {
    Free,
    Used,
    Boundary,
}

/// A physical page bitmap allocator
#[repr(C, packed)]
pub struct HeapBitmap {
    map: [BlockStatus; KHEAP_NUM_BLOCKS],
    arena: usize,
}

impl HeapBitmap {
    pub fn new(arena: usize) -> HeapBitmap {
        HeapBitmap {
            map: [BlockStatus::Free; KHEAP_NUM_BLOCKS],
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
        (self.arena + (KHEAP_BLOCK_SIZE * block_num)) as *mut u8
    }

    pub fn malloc(&mut self, n: usize) -> Option<*mut u8> {
        let mut alloc_start = 0;
        let mut found = 0;

        for i in 0..KHEAP_NUM_BLOCKS {
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
        let mut pos = ((ptr as usize) - self.arena) / KHEAP_BLOCK_SIZE;
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
pub fn get_bitmap() -> &'static mut HeapBitmap {
    unsafe {
        let ptr = KERNEL_END as *mut HeapBitmap;
        ptr.as_mut().expect("Kernel heap is null")
    }
}

/// Initialize a single reserved page for use as the kernel
/// heap bitmap
#[no_mangle]
pub unsafe extern "C" fn kmalloc_init() {
    let root = get_bitmap();

    // Align to the nearest page, leaving space for the bitmap
    let kheap_begin = (((KERNEL_END + size_of!(HeapBitmap)) / L0_PAGE_SIZE) + 1) * L0_PAGE_SIZE;

    // Initialize the bitmap
    *root = HeapBitmap::new(kheap_begin);
}

/// Alllocate physical pages from the kernel heap
pub fn kmalloc(pages: usize) -> Option<*mut u8> {
    let map = get_bitmap();
    map.malloc(pages)
}

/// Free an allocations pages
pub fn kfree(ptr: *mut u8) {
    let map = get_bitmap();
    map.free(ptr);
}
