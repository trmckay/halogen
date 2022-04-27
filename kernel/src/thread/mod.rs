use crate::{arch::Context, mem::Stack, prelude::*};

mod executor;
mod state;

pub use executor::{handoff, join, resume, spawn, tid, timer_event, yld};

pub type ThreadFunction = extern "C" fn(usize) -> usize;
pub type ThreadShim = extern "C" fn(ThreadFunction, usize);
pub type ThreadId = usize;


#[derive(Debug, Copy, Clone)]
pub enum ThreadState {
    Running,
    Blocked,
    Ready,
    Finished,
}

impl Default for ThreadState {
    fn default() -> ThreadState {
        ThreadState::Ready
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ThreadType {
    Kernel,
}


#[derive(Debug, Copy, Clone)]
pub enum ThreadError {
    NoSuchThread(usize),
    ThreadAllocation,
}

/// An execution context in the main kernel address space
///
/// A kernel thread entry point must be of the type
/// `extern "C" fn(usize) -> usize`
///
/// The first argument can be cast to a pointer for generic arguments
///
/// The return value is an exit code
#[derive(Debug, Clone)]
pub struct Thread {
    pub tid: usize,
    pub typ: ThreadType,
    pub context: Context,
    pub entry: extern "C" fn(usize) -> usize,
    pub arg: usize,
    pub state: ThreadState,
    pub ret: usize,
    stack: Stack,
}

impl Thread {
    pub fn new(
        tid: usize,
        typ: ThreadType,
        entry: extern "C" fn(usize) -> usize,
        arg: usize,
    ) -> Option<Thread> {
        let stack = Stack::new(16)?;
        let thread = Thread {
            tid,
            typ,
            entry,
            arg,
            stack,
            ret: 0,
            state: ThreadState::default(),
            context: Context::default(),
        };
        Some(thread)
    }

    /// # Safety
    ///
    /// * `ctx` must point to a valid `Context`
    /// * `ctx.env` must match the type of `self` (e.g. `Supervisor` for
    ///   `Kernel`)
    pub unsafe fn save_context(&mut self, ctx: *const Context) {
        self.context = transmute_copy(&*ctx);
    }

    pub fn stack(&self) -> *mut u8 {
        self.stack.top()
    }

    pub fn context_ptr(&mut self) -> *mut Context {
        ptr::addr_of_mut!(self.context)
    }
}
