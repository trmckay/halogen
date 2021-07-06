use super::Program;
use crate::memory::AddressSpace;

pub struct Context {
    addr_space: AddressSpace,
}

impl Context {
    /// Allocate some space for a context.
    pub fn new(prog: &Program) -> Self {
        Context {
            addr_space: AddressSpace::new(prog),
        }
    }

    /// Load the saved registers back onto the CPU.
    pub fn load(&self) {}

    /// Save the CPU registers to memory.
    pub fn save(&mut self) {}
}
