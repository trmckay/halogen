use super::Context;
use crate::memory::AddressSpace;
use crate::program::Program;

pub enum ProcessState {
    Ready,
    Running,
    Blocked,
}

/// A process represents a program running within its own context.
pub struct Process {
    id: u32,
    state: ProcessState,
    addr_space: AddressSpace,
    context: Option<Context>,
}

impl Process {
    /// Initialize (but do not start) a new process.
    pub fn new(prog: &Program) -> Self {
        Process {
            id: 0,
            state: ProcessState::Ready,
            addr_space: AddressSpace::new(),
            context: None,
        }
    }

    pub fn start(&mut self) {}

    pub fn sleep(&mut self) {}

    pub fn kill(&mut self) {}
}
