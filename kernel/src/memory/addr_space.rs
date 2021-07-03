pub struct AddressSpace {
    phys_addr: usize,
    size: usize,
}

impl AddressSpace {
    pub fn new() -> Self {
        AddressSpace {
            phys_addr: 0,
            size: 0,
        }
    }
}
