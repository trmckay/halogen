use halogen_common::mem::KIB;

use crate::{
    arch::{Context, Privilege},
    error::KernelError,
    mem::Stack,
};

pub type ThreadFunction = extern "C" fn(usize) -> usize;
pub type ThreadShim = extern "C" fn(ThreadFunction, usize);
pub type ThreadId = usize;

pub const THREAD_STACK_SIZE: usize = 64 * KIB;

/// Possible states for a thread.
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
    /// Create a new thread structure for calling a function in a new kernel
    /// thread.
    pub fn new_kernel(
        tid: usize,
        entry: ThreadFunction,
        arg: usize,
    ) -> Result<Thread, KernelError> {
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
        Ok(thread)
    }

    /// Save the context stored at the pointer in the thread structure.
    ///
    /// # Safety
    ///
    /// - `ctx` must point to a valid `Context`.
    /// - `ctx.env` must match the type of `self` (e.g. `Supervisor` for
    ///   `Kernel`).
    pub unsafe fn save_context(&mut self, ctx: *const Context) {
        self.context = core::mem::transmute_copy(&*ctx);
    }

    /// Get a pointer to the top of the thread's stack.
    pub fn stack(&self) -> *mut u8 {
        self.stack.top()
    }

    /// Get a pointer to the thread's context.
    pub fn context_ptr(&mut self) -> *mut Context {
        core::ptr::addr_of_mut!(self.context)
    }
}
