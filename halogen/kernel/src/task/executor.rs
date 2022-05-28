use alloc::{boxed::Box, collections::BTreeMap};

use halogen_common::sched::{RoundRobinScheduler, TaskScheduler};
use lazy_static::lazy_static;
use spin::Mutex;

use super::thread::{Thread, ThreadFunction, ThreadId, ThreadState};
use crate::{arch::Context, critical_section, error::KernelError, irq, kerror, log::*, sbi};

/// Number of time slices each thread gets before being descheduled.
pub const DEFAULT_QUANTA_LIMIT: usize = 4;
/// Length of a single time slice.
pub const DEFAULT_QUANTUM_US: usize = 250_000;

lazy_static! {
    static ref EXECUTOR: Mutex<Executor> = Mutex::new(Executor::default());
}

/// Wraps a thread function to capture the return value and cleanly exit.
extern "C" fn thread_shim(entry: ThreadFunction, arg: usize) {
    let ret = entry(arg);

    info!("Thread {} finished (ret={})", tid(), ret);

    critical_section! {
        EXECUTOR.lock().register_return(ret);
    }

    sbi::timer::set(0).expect("failed to set timer");
}

/// Add a kernel thread to the executor pool.
pub fn spawn(entry: ThreadFunction, arg: usize) -> Result<usize, KernelError> {
    critical_section! {
        match EXECUTOR.lock().spawn(entry, arg) {
            Err(why) => {
                error!("Failed to spawn thread: {:?}", why);
                Err(why)
            }
            Ok(tid) => {
                info!("Spawn thread {} ({:p})({})", tid, entry, arg);
                Ok(tid)
            }
        }
    }
}

/// Give up remaining quanta.
pub fn yld() {
    critical_section! {
        EXECUTOR.lock().yld();
        sbi::timer::set(0).expect("failed to set timer")
    };
}

/// Wait for a thread to complete and return its result.
pub fn join(tid: usize) -> Result<usize, KernelError> {
    loop {
        let ret = critical_section! {
            EXECUTOR.lock().reap(tid)
        };

        match ret {
            Err(why) => return Err(why),
            Ok(Some(ret)) => return Ok(ret),
            _ => yld(),
        }
    }
}


/// Save the context for the current thread and return the next context.
///
/// # Safety
///
/// - `saved_ctx` must not be null.
/// - `saved_ctx` must point to a valid Context.
/// - This should probably only be called when returning from a trap handler.
pub unsafe fn resume(saved_ctx: &Context) -> *const Context {
    EXECUTOR.lock().resume(saved_ctx)
}

/// Handoff control to the thread executor.
pub fn handoff(entry: ThreadFunction, arg: usize) -> ! {
    info!("Handing off control to thread executor");
    spawn(entry, arg).expect("failed to spawn handoff thread");

    irq::enable_timer();
    sbi::timer::set(0).expect("failed to set timer");

    panic!("returned from executor handoff")
}

/// Register a timer event.
pub fn timer_event() {
    let mut executor = EXECUTOR.lock();
    executor.register_quantum();
    sbi::timer::set(executor.quantum_len).expect("failed to set timer");
}

/// Get the ID of the calling thread.
pub fn tid() -> usize {
    EXECUTOR
        .lock()
        .scheduler
        .current()
        .expect("no thread running")
}

/// Coordinates execution and scheduling of processes and kernel threads.
pub struct Executor {
    scheduler: Box<dyn TaskScheduler<Handle = ThreadId>>,
    threads: BTreeMap<ThreadId, Thread>,
    quanta_limit: usize,
    quanta: BTreeMap<ThreadId, usize>,
    quantum_len: usize,
}

impl Default for Executor {
    fn default() -> Executor {
        Executor {
            quanta_limit: DEFAULT_QUANTA_LIMIT,
            quantum_len: DEFAULT_QUANTUM_US,
            threads: BTreeMap::default(),
            quanta: BTreeMap::default(),
            scheduler: Box::new(RoundRobinScheduler::default()),
        }
    }
}

impl Executor {
    /// Create a new thread.
    fn spawn(&mut self, entry: ThreadFunction, arg: usize) -> Result<usize, KernelError> {
        let tid = match self.scheduler.add_new() {
            Some(tid) => tid,
            None => {
                return kerror!(
                    KernelError::ThreadCreate,
                    kerror!(KernelError::SchedulerAdd)
                )
                .into()
            }
        };

        match Thread::new_kernel(tid, entry, arg) {
            Ok(mut thread) => {
                // Prepare the context to enter at the thread shim holding the correct arguments
                thread
                    .context
                    .prepare(thread.stack(), thread_shim, thread.entry, thread.arg);

                self.threads.insert(tid, thread);
                self.quanta.insert(tid, 0);

                Ok(tid)
            }
            Err(why) => kerror!(KernelError::ThreadCreate, why).into(),
        }
    }

    /// Yield the caller's remaining time.
    fn yld(&mut self) {
        if let Some(tid) = self.scheduler.current() {
            self.scheduler.yld(tid);
            self.quanta.insert(tid, self.quanta_limit);
        }
    }

    /// Clean up a thread and return its value.
    fn reap(&mut self, tid: usize) -> Result<Option<usize>, KernelError> {
        match self.threads.get(&tid) {
            None => kerror!(KernelError::ThreadReap, kerror!(KernelError::NoSuchThread)).into(),
            Some(thread) => {
                Ok(match thread.state {
                    ThreadState::Finished => {
                        trace!("Reap thread {}", thread.tid);
                        let tid = thread.tid;
                        let ret = Some(thread.ret);
                        self.quanta.remove(&tid);
                        self.threads.remove(&tid);
                        ret
                    }
                    _ => None,
                })
            }
        }
    }

    /// Returns the ID and a pointer to the `Context` of the next thread.
    fn resume<'a>(&'a mut self, saved_ctx: &'a Context) -> &'a Context {
        match self.current() {
            // Coming from a running thread
            Some(current) => {
                let state = current.state;
                let time_reached = self.time_reached();

                match (state, time_reached) {
                    // Thread is running but out of quanta
                    (ThreadState::Running, true) => {
                        // Add the thread to the back of the queue if it's running
                        // Update the current thread
                        let thread = self.current_mut().unwrap();
                        thread.state = ThreadState::Ready;
                        unsafe {
                            thread.save_context(saved_ctx);
                        }

                        // Move to the next thread
                        let next = self.update_and_get_next();
                        trace!("Swap to thread {}", next.tid);
                        &next.context
                    }
                    // Thread is running and still has time left
                    (ThreadState::Running, false) => {
                        trace!("Resuming thread");
                        saved_ctx
                    }
                    // Invalid state
                    (state, _) => {
                        panic!("{:?} thread cannot run", state)
                    }
                }
            }

            // First time polling for next thread or a thread just finished
            None => {
                let next = self.update_and_get_next();
                trace!("Swap to thread {}", next.tid);
                &next.context
            }
        }
    }

    /// Returns whether the current thread has reached its quanta limit, false
    /// if the limit is not reached or no thread is running.
    fn time_reached(&self) -> bool {
        match self.scheduler.current() {
            Some(tid) => {
                *self
                    .quanta
                    .get(&tid)
                    .unwrap_or_else(|| panic!("no quanta for thread {}", tid))
                    >= self.quanta_limit
            }
            None => false,
        }
    }

    /// Call once per timer event to increment the current thread's quanta.
    fn register_quantum(&mut self) {
        if let Some(tid) = self.scheduler.current() {
            *self
                .quanta
                .get_mut(&tid)
                .unwrap_or_else(|| panic!("no quanta for thread {}", tid)) += 1;
        }
    }

    /// Returns a reference to the currently running thread.
    fn current(&self) -> Option<&Thread> {
        self.scheduler
            .current()
            .and_then(|tid| self.threads.get(&tid))
    }

    /// Returns a mutable reference to the currently running thread.
    fn current_mut(&mut self) -> Option<&mut Thread> {
        self.scheduler
            .current()
            .and_then(move |tid| self.threads.get_mut(&tid))
    }

    /// Get the next thread from the scheduler and update its state.
    fn update_and_get_next(&mut self) -> &mut Thread {
        let next_tid = self
            .scheduler
            .next()
            .expect("scheduler returned no next thread");

        let thread = self
            .threads
            .get_mut(&next_tid)
            .unwrap_or_else(|| panic!("no such thread {}", next_tid));

        thread.state = ThreadState::Running;
        thread
    }

    /// Register a thread as having returned and save its return value; keep it
    /// around until it is joined and reaped.
    fn register_return(&mut self, ret: usize) {
        let curr_tid = self
            .scheduler
            .current()
            .expect("scheduler returned no current thread");

        self.scheduler.complete(curr_tid);

        let curr = self
            .threads
            .get_mut(&curr_tid)
            .unwrap_or_else(|| panic!("no such thread {}", curr_tid));

        curr.ret = ret;
        curr.state = ThreadState::Finished;
    }
}
