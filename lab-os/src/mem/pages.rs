use super::NULL_PTR_MUT;
use crate::{k_heap_begin, k_heap_size, println};

extern "C" {
    fn write_header(header: *mut u8, size: usize, next: *mut u8);
}

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

pub fn heap_init() {
    unsafe {
        let header: &mut PageHeader = PageHeader::from_ptr(k_heap_begin!());
        header.size = k_heap_size!() - 8;
        header.next = NULL_PTR_MUT;
    }
}

pub fn heap_dump() {
    unsafe {
        let mut header: &mut PageHeader = PageHeader::from_ptr(k_heap_begin!());

        loop {
            println!("Heap header: {:p}, {} KB", header, header.size / 1024);
            match header.next() {
                None => {
                    break;
                }
                Some(next) => header = next,
            }
        }
    }
}
