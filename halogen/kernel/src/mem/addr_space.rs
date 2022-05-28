use crate::{error::KernelError, mem::paging::PageTable};

/// A virtual address space isolated to a single process.
pub struct AddressSpace<'a> {
    id: usize,
    root: &'a mut PageTable,
}

impl<'a> AddressSpace<'a> {
    /// Create a new `AddressSpace`.
    pub fn new(_id: usize) -> Option<AddressSpace<'a>> {
        todo!()
    }

    /// Populates memory regions with the loadable contents of an ELF
    /// executable.
    pub fn load_elf(&mut self, _bytes: &[u8]) -> Result<(), KernelError> {
        todo!()
    }
}
