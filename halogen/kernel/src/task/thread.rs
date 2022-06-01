use halogen_common::mem::{Segment, VirtualAddress, KIB, MIB};

use super::{process::Process, yld};
use crate::{
    arch::{Context, Privilege},
    error::KernelResult,
    mem::{paging::get_satp, Stack},
    task::executor::exit,
};

pub type ThreadFunction = extern "C" fn(usize) -> isize;
pub type ThreadShim = extern "C" fn(ThreadFunction, usize);

pub const THREAD_STACK_SIZE: usize = MIB;

const USER_START: VirtualAddress = VirtualAddress(0x1000);
const USER_STACK_TOP: VirtualAddress = VirtualAddress(0x8000_0000);
const USER_STACK_SIZE: usize = 64 * KIB;

/// Wraps a thread function to capture the return value and cleanly exit.
extern "C" fn thread_shim(entry: ThreadFunction, arg: usize) {
    let ret = entry(arg);
    exit(ret);
    yld();
}

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

pub enum Thread {
    Kernel(KernelThread),
    User(UserThread),
}

impl core::fmt::Display for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Thread::User(ut) => write!(f, "(PID={}, TID={})", ut.pid, ut.tid),
            Thread::Kernel(kt) => write!(f, "(kernel, TID={})", kt.tid),
        }
    }
}

/// A thread that runs in kernel space and does not belong to a process.
pub struct KernelThread {
    pub tid: usize,
    pub context: Context,
    pub entry: ThreadFunction,
    pub arg: usize,
    pub state: ThreadState,
    pub exit: Option<isize>,
    stack: Stack,
}

impl Thread {
    pub fn save_context(&mut self, ctx: &Context) {
        unsafe {
            match self {
                Thread::User(ut) => ut.context = core::mem::transmute_copy(ctx),
                Thread::Kernel(kt) => kt.context = core::mem::transmute_copy(ctx),
            };
        }
    }

    pub fn context(&self) -> &Context {
        match self {
            Thread::User(ut) => &ut.context,
            Thread::Kernel(kt) => &kt.context,
        }
    }

    pub fn state(&self) -> ThreadState {
        match self {
            Thread::User(ut) => ut.state,
            Thread::Kernel(kt) => kt.state,
        }
    }

    pub fn set_state(&mut self, state: ThreadState) {
        match self {
            Thread::User(ut) => ut.state = state,
            Thread::Kernel(kt) => kt.state = state,
        };
    }

    pub fn tid(&self) -> usize {
        match self {
            Thread::User(ut) => ut.tid,
            Thread::Kernel(kt) => kt.tid,
        }
    }

    pub fn exit_status(&self) -> Option<isize> {
        match self.state() {
            ThreadState::Finished => {
                match self {
                    Thread::User(ut) => ut.exit,
                    Thread::Kernel(kt) => kt.exit,
                }
            }
            _ => None,
        }
    }

    pub fn exit(&mut self, status: isize) {
        match self {
            Thread::User(ut) => ut.exit = Some(status),
            Thread::Kernel(kt) => kt.exit = Some(status),
        };
        self.set_state(ThreadState::Finished);
    }

    pub fn pid(&self) -> Option<usize> {
        match self {
            Thread::Kernel(_) => None,
            Thread::User(ut) => Some(ut.pid),
        }
    }
}

impl KernelThread {
    /// Create a new thread structure for calling a function in a new kernel
    /// thread.
    pub fn try_new(tid: usize, entry: ThreadFunction, arg: usize) -> KernelResult<KernelThread> {
        let stack = Stack::try_new_kernel(THREAD_STACK_SIZE)?;
        let mut thread = KernelThread {
            tid,
            entry,
            arg,
            stack,
            exit: None,
            state: ThreadState::default(),
            context: Context::new_kernel(),
        };

        thread.context.pc = thread_shim as usize;
        thread.context.gp_regs[9] = thread.entry as usize;
        thread.context.gp_regs[10] = thread.arg;
        thread.context.gp_regs[1] = thread.stack() as usize;

        Ok(thread)
    }

    /// Get a pointer to the top of the thread's stack.
    pub fn stack(&self) -> *mut u8 {
        self.stack.top()
    }
}

pub struct UserThread {
    pub pid: usize,
    pub tid: usize,
    pub state: ThreadState,
    pub context: Context,
    pub exit: Option<isize>,
    stack: Stack,
}

impl UserThread {
    pub fn try_new(tid: usize, parent: &Process) -> KernelResult<UserThread> {
        let stack = unsafe {
            Stack::try_new_user(
                Segment::from_size(USER_STACK_TOP - USER_STACK_SIZE, USER_STACK_SIZE),
                USER_STACK_SIZE,
            )?
        };

        let ctx = Context {
            pc: USER_START.into(),
            satp: get_satp(parent.pid as u16, &parent.space.root),
            prv: Privilege::User,
            ..Default::default()
        };

        Ok(UserThread {
            tid,
            pid: parent.pid,
            state: ThreadState::default(),
            context: ctx,
            stack,
            exit: None,
        })
    }
}
