use halogen_common::mem::KIB;

use crate::{
    arch::{Context, Privilege},
    mem::Stack,
};

pub type ThreadFunction = extern "C" fn(usize) -> usize;
pub type ThreadShim = extern "C" fn(ThreadFunction, usize);
pub type ThreadId = usize;

pub const THREAD_STACK_SIZE: usize = 64 * KIB;

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
pub enum ThreadError {
    NoSuchThread(usize),
    AllocationFailed,
    SchedulerError,
}

/// An execution context in the main kernel address space
///
/// A kernel thread entry point must be of the type
/// `extern "C" fn(usize) -> usize`
///
/// The first argument can be cast to a pointer for generic arguments
///
/// The return value is an exit code
pub struct Thread {
    pub tid: ThreadId,
    pub prv: Privilege,
    pub context: Context,
    pub entry: extern "C" fn(usize) -> usize,
    pub arg: usize,
    pub state: ThreadState,
    pub ret: usize,
    stack: Stack,
}

impl Thread {
    pub fn new_kernel(
        tid: usize,
        entry: extern "C" fn(usize) -> usize,
        arg: usize,
    ) -> Option<Thread> {
        let stack = Stack::new(THREAD_STACK_SIZE)?;
        let thread = Thread {
            tid,
            prv: Privilege::Supervisor,
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
        self.context = core::mem::transmute_copy(&*ctx);
    }

    pub fn stack(&self) -> *mut u8 {
        self.stack.top()
    }

    pub fn context_ptr(&mut self) -> *mut Context {
        core::ptr::addr_of_mut!(self.context)
    }
}
