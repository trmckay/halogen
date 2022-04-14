use super::{Thread, ThreadFunction, ThreadState, ThreadType};
use crate::{arch::Context, irq, prelude::*, println_unsafe, sbi};

pub const TIMER_FREQ_HZ: usize = 10_000_000;

pub const DEFAULT_QUANTA_LIMIT: usize = 4;
pub const DEFAULT_QUANTUM_US: isize = 250_000;

pub const DEFAULT_QUANTUM_CYCLES: isize = DEFAULT_QUANTUM_US * TIMER_FREQ_HZ as isize / 1_000_000;

lazy_static! {
    static ref EXECUTOR: Mutex<Executor> = Mutex::new(Executor::default());
}

extern "C" fn test_thread(i: usize) -> usize {
    loop {
        unsafe {
            println_unsafe!("hello({})", i);
        }
    }
}

extern "C" fn thread_shim(entry: ThreadFunction, arg: usize) {
    let ret = entry(arg);

    {
        let mut executor = EXECUTOR.lock();
        let curr = executor.current_mut().expect("executor: not running");

        curr.ret = ret;
        curr.state = ThreadState::Finished;
    }

    yld();
}

/// Add a kernel thread to the executor pool
pub fn spawn(entry: ThreadFunction, arg: usize) -> Option<usize> {
    EXECUTOR.lock().spawn(entry, arg)
}

/// Save the context for the current thread and return the next context
///
/// # Safety
///
/// * `saved_ctx` must not be null
/// * `saved_ctx` must point to a valid Context
pub unsafe fn resume(saved_ctx: *const Context) -> *const Context {
    EXECUTOR.lock().resume(saved_ctx)
}

/// Elect to block a thread from kernel code
pub fn yld() {
    EXECUTOR.lock().yld();
    sbi::set_timer(0).expect("executor: failed to set timer");
}

/// Handoff control to the thread executor
pub fn handoff(entry: ThreadFunction, arg: usize) -> ! {
    let tid = spawn(entry, arg).expect("executor: failed to spawn handoff thread");

    EXECUTOR.lock().queue.push_back(tid);

    irq::enable_timer();
    sbi::set_timer(0).expect("executor: failed to set timer");

    panic!("executor: returned from handoff thread")
}

/// Register a timer event
pub fn timer_event() {
    let mut executor = EXECUTOR.lock();
    executor.add_quantum();
    sbi::set_timer(executor.quantum_len).expect("executor: failed to set timer");
}

/// Get the ID of the calling thread
pub fn tid() -> usize {
    EXECUTOR.lock().current.expect("executor: not running")
}

pub struct Executor {
    current: Option<usize>,
    tids: usize,
    threads: BTreeMap<usize, Thread>,
    queue: VecDeque<usize>,
    quanta_limit: usize,
    quanta: BTreeMap<usize, usize>,
    quantum_len: isize,
}

impl Default for Executor {
    fn default() -> Executor {
        Executor {
            current: None,
            tids: 1,
            quanta_limit: DEFAULT_QUANTA_LIMIT,
            quantum_len: DEFAULT_QUANTUM_CYCLES,
            threads: BTreeMap::default(),
            queue: VecDeque::default(),
            quanta: BTreeMap::default(),
        }
    }
}

impl Executor {
    fn spawn(&mut self, entry: ThreadFunction, arg: usize) -> Option<usize> {
        let tid = self.tids;
        self.tids += 1;

        let mut thread = Thread::new(tid, ThreadType::Kernel, entry, arg)?;

        // Prepare the context to enter at the thread shim holding the correct arguments
        thread
            .context
            .prepare(thread.stack(), thread_shim, thread.entry, thread.arg);

        self.threads.insert(tid, thread);
        self.quanta.insert(tid, 0);
        self.queue.push_back(tid);

        Some(tid)
    }

    fn yld(&mut self) {
        if let Some(thread) = self.current_mut() {
            if let ThreadState::Running = thread.state {
                thread.state = ThreadState::Blocked;
            }
        }
    }

    fn set_next(&mut self) -> &mut Thread {
        let next = self.queue.pop_front().expect("executor: no viable threads");
        self.current = Some(next);
        let thread = self
            .threads
            .get_mut(&next)
            .expect("executor: thread does not exist");
        thread.state = ThreadState::Running;
        thread
    }

    /// Returns a pointer to the `Context` of the next thread to be run
    fn resume(&mut self, saved_ctx: *const Context) -> *const Context {
        match self.current() {
            // Coming from a running thread
            Some(current) => {
                let tid = current.tid;
                let state = current.state;
                let time_reached = self.time_reached();

                match (state, time_reached) {
                    (ThreadState::Finished, _) | (ThreadState::Running, true) => {
                        // Update the current thread
                        self.queue.push_back(tid);
                        let thread = self.current_mut().unwrap();
                        thread.state = ThreadState::Ready;
                        unsafe {
                            thread.save_context(saved_ctx);
                        }

                        // Move to the next thread
                        self.set_next().context_ptr()
                    }
                    // Thread still has time left
                    (ThreadState::Running, false) => saved_ctx,
                    // Invalid states
                    (ThreadState::Blocked | ThreadState::Ready, _) => {
                        panic!("executor: a {:?} thread cannot run", current.state)
                    }
                }
            }
            // First time polling for next thread
            None => self.set_next().context_ptr(),
        }
    }

    fn add_quantum(&mut self) {
        if let Some(tid) = self.current {
            *self
                .quanta
                .get_mut(&tid)
                .expect("executor: no quanta for thread") += 1;
        }
    }

    fn started(&self) -> bool {
        self.current.is_some()
    }

    /// Returns a reference to the currently running thread
    fn current(&self) -> Option<&Thread> {
        self.current.and_then(|tid| self.threads.get(&tid))
    }

    /// Returns a mutable reference to the currently running thread
    fn current_mut(&mut self) -> Option<&mut Thread> {
        self.current.and_then(move |tid| self.threads.get_mut(&tid))
    }

    /// Returns whether the current thread has reached its quanta limit
    fn time_reached(&self) -> bool {
        *self
            .quanta
            .get(&self.current().expect("executor: not running").tid)
            .expect("executor: no quanta for thread")
            >= self.quanta_limit
    }
}
