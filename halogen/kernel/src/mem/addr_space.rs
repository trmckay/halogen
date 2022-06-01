use crate::mem::paging::PageTable;

/// A virtual address space isolated to a single process.
#[derive(Debug, Clone, Copy, Default)]
pub struct AddressSpace {
    pub id: usize,
    pub root: PageTable,
}

impl AddressSpace {
    /// Create a new `AddressSpace` populated with the kernel mappings.
    pub fn new(id: usize) -> AddressSpace {
        AddressSpace {
            id,
            root: PageTable::from_kernel_root(),
        }
    }
}
