use super::Context;
use super::Program;

// Different states a process can exist in.
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
}

/// A process represents a program running within its own context.
pub struct Process {
    id: u32,
    state: ProcessState,
    context: Context,
}

impl Process {
    /// Initialize (but do not start) a new process.
    pub fn new(prog: &Program) -> Self {
        Process {
            id: 0,
            state: ProcessState::Ready,
            context: Context::new(prog),
        }
    }

    /// Load a process' context and resume execution.
    pub fn start(&mut self) {}

    /// Save a process' context and pause execution.
    pub fn sleep(&mut self) {}

    /// Kill a process.
    pub fn kill(&mut self) {}
}
