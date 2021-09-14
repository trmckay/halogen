pub use crate::mem::page::PageAllocator;
use crate::mem::NULL_PTR_MUT;
use crate::{k_heap_begin, k_heap_size};

/// A header that proceeds a free block of memory on the heap. It records
/// the size and points to the next free block.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PageHeader {
    next: *mut u8,
    pub size: usize,
}

impl PageHeader {
    /// Cast a raw pointer to a page header. This is obviously an unsafe operation,
    /// as it assumes the pointer is valid.
    pub unsafe fn from_ptr<'a>(ptr: *mut u8) -> &'a mut PageHeader {
        &mut *(ptr as *mut PageHeader)
    }

    /// Get the next free heap block pointed to by this header.
    pub fn next<'a>(&mut self) -> Option<&'a mut PageHeader> {
        unsafe {
            match self.next {
                NULL_PTR_MUT => None,
                _ => Some(&mut *(self.next as *mut PageHeader)),
            }
        }
    }
}

pub struct LinkedListAllocator;

impl PageAllocator for LinkedListAllocator {
    fn init(&self) {
        unsafe {
            let header: &mut PageHeader = PageHeader::from_ptr(k_heap_begin!());
            header.size = k_heap_size!() - 8;
            header.next = NULL_PTR_MUT;
        }
    }

    fn alloc(&self, num: usize) -> Option<*mut u8> {
        let bytes_requested = num * 4096;
        let mut header: &mut PageHeader;
        unsafe {
            header = PageHeader::from_ptr(k_heap_begin!());
        }
        loop {
            if header.size >= bytes_requested {
                header.size -= bytes_requested;
            }
            header = match header.next() {
                None => return None,
                Some(next) => next,
            }
        }
    }

    fn free(&self, page: *mut u8) {}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test_case]
    fn test_linked_list_page_alloc_init() {
        assert!(true);
        let allocator = LinkedListAllocator;
        LinkedListAllocator.init();

        let header;
        unsafe {
            header = PageHeader::from_ptr(k_heap_begin!());
        }
        assert_eq!(k_heap_size!() - 8, header.size);
        assert!(header.next().is_none());
    }
}
