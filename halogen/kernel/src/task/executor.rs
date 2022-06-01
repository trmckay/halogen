use alloc::{boxed::Box, collections::BTreeMap};

use halogen_common::sched::{RoundRobinScheduler, TaskScheduler};
use lazy_static::lazy_static;
use spin::Mutex;

use super::{
    process::Process,
    thread::{KernelThread, Thread, ThreadFunction, ThreadState},
};
use crate::{
    arch::Context,
    critical_section,
    error::{KernelError, KernelResult},
    irq, kerror,
    log::*,
    mem::paging::KERNEL_ASID,
    sbi::timer,
};

/// Number of time slices each thread gets before being descheduled.
pub const DEFAULT_QUANTA_LIMIT: usize = 4;
/// Length of a single time slice.
pub const DEFAULT_QUANTUM_US: usize = 250_000;

lazy_static! {
    static ref EXECUTOR: Mutex<Executor> = Mutex::new(Executor::default());
}

/// Add a kernel thread to the executor pool.
pub fn spawn(entry: ThreadFunction, arg: usize) -> KernelResult<usize> {
    critical_section!({
        match EXECUTOR.lock().spawn_kernel(entry, arg) {
            Err(why) => {
                error!("Failed to spawn thread: {:?}", why);
                Err(why)
            }
            Ok(tid) => {
                info!("Spawn thread {} ({:p})({})", tid, entry, arg);
                Ok(tid)
            }
        }
    })
}

/// Give up remaining quanta.
pub fn yld() {
    critical_section! {{
        EXECUTOR.lock().yld();
        timer::set(0);
    }};
}

pub fn exit(status: isize) {
    critical_section! {{
        EXECUTOR.lock().exit(status);
    }};
    yld();
}

/// Spawn a process and return the PID and main thread's TID.
pub fn exec(elf: &[u8]) -> KernelResult<(usize, usize)> {
    let (pid, tid) = critical_section!({
        let mut executor = EXECUTOR.lock();
        let pid = executor.get_pid();
        let mut proc = Process::try_from_elf(pid, elf)?;

        let main_tid = executor.get_tid();
        let main = proc.create_main(main_tid)?;

        executor.add_thread(main_tid, Thread::User(main));
        executor.processes.insert(pid, proc);

        Ok((pid, main_tid))
    })?;

    trace!("Create process {} with main thread {}", pid, tid);

    yld();

    Ok((pid, tid))
}

/// Wait for a thread to complete and return its result.
pub fn join(tid: usize) -> KernelResult<isize> {
    loop {
        let completed = critical_section!({ EXECUTOR.lock().is_complete(tid) });
        if completed {
            return critical_section!({
                EXECUTOR
                    .lock()
                    .reap(tid)
                    .map(|opt| opt.expect("complete thread has no exit status"))
            });
        } else {
            yld()
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
    timer::set(0);

    panic!("returned from executor handoff")
}

/// Register a timer event.
pub fn timer_event() {
    let mut executor = EXECUTOR.lock();
    executor.register_quantum();
    timer::set(executor.quantum_len);
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
struct Executor {
    tid_counter: usize,
    scheduler: Box<dyn TaskScheduler<Handle = usize>>,
    threads: BTreeMap<usize, Thread>,
    quanta_limit: usize,
    quanta: BTreeMap<usize, usize>,
    quantum_len: usize,
    processes: BTreeMap<usize, Process>,
    pid_counter: usize,
}

impl Default for Executor {
    fn default() -> Executor {
        Executor {
            tid_counter: 0,
            pid_counter: KERNEL_ASID as usize + 1,
            quanta_limit: DEFAULT_QUANTA_LIMIT,
            quantum_len: DEFAULT_QUANTUM_US,
            threads: BTreeMap::default(),
            processes: BTreeMap::default(),
            quanta: BTreeMap::default(),
            scheduler: Box::new(RoundRobinScheduler::default()),
        }
    }
}

impl Executor {
    /// Create a new thread.
    fn spawn_kernel(&mut self, entry: ThreadFunction, arg: usize) -> KernelResult<usize> {
        let tid = self.get_tid();
        self.scheduler.add_new(tid);

        // Prepare the context to enter at the thread shim holding the correct arguments
        let thread = Thread::Kernel(KernelThread::try_new(tid, entry, arg)?);

        self.threads.insert(tid, thread);
        self.quanta.insert(tid, 0);

        Ok(tid)
    }

    fn get_tid(&mut self) -> usize {
        let tid = self.tid_counter;
        self.tid_counter += 1;
        tid
    }

    fn get_pid(&mut self) -> usize {
        let pid = self.pid_counter;
        self.pid_counter += 1;
        pid
    }

    /// Yield the caller's remaining time.
    fn yld(&mut self) {
        if let Some(tid) = self.scheduler.current() {
            self.scheduler.yld(tid);
            self.quanta.insert(tid, self.quanta_limit);
        }
    }

    /// Returns true if the thread exists and is complete.
    fn is_complete(&self, tid: usize) -> bool {
        match self.threads.get(&tid) {
            Some(thread) => matches!(thread.state(), ThreadState::Finished),
            None => false,
        }
    }

    /// Clean up a thread and return its value.
    fn reap(&mut self, tid: usize) -> KernelResult<Option<isize>> {
        match self.threads.get(&tid) {
            None => kerror!(KernelError::NoSuchThread).into(),
            Some(thread) => {
                trace!("Reap thread {}", thread);

                let tid = thread.tid();
                let ret = thread.exit_status();

                // Clean up process if this was the main thread.
                if let Thread::User(ut) = thread {
                    let (pid, main_tid) = {
                        let parent = self
                            .processes
                            .get_mut(&ut.pid)
                            .expect("thread has no parent");

                        parent.tids.retain(|&tid| tid != thread.tid());
                        (parent.pid, parent.main_tid)
                    };

                    if thread.tid() == main_tid {
                        info!("Clean up process {}", pid);
                        self.processes.remove(&pid);
                    }
                }

                self.threads.remove(&tid);
                self.quanta.remove(&tid);

                Ok(ret)
            }
        }
    }

    /// Returns the ID and a pointer to the `Context` of the next thread.
    fn resume<'a>(&'a mut self, saved_ctx: &'a Context) -> &'a Context {
        match self.get_current() {
            // Coming from a running thread
            Some(current) => {
                let state = current.state();
                let time_reached = self.time_reached();

                match (state, time_reached) {
                    // Thread is running but out of quanta
                    (ThreadState::Running, true) => {
                        // Add the thread to the back of the queue if it's running
                        // Update the current thread
                        let thread = self.current_mut().unwrap();
                        thread.set_state(ThreadState::Ready);
                        thread.save_context(saved_ctx);

                        // Move to the next thread
                        let next = self.update_and_get_next();
                        trace!("Swap to thread {}", next.tid());
                        next.context()
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
                trace!("Swap to thread {}", next.tid());
                next.context()
            }
        }
    }

    fn add_thread(&mut self, tid: usize, thread: Thread) {
        self.threads.insert(tid, thread);
        self.quanta.insert(tid, 0);
        self.scheduler.add_new(tid);
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
    fn get_current(&self) -> Option<&Thread> {
        self.scheduler
            .current()
            .and_then(|tid| self.threads.get(&tid))
    }

    /// Get a mutable reference to a thread with an ID.
    fn get_mut(&mut self, tid: usize) -> Option<&mut Thread> {
        self.threads.get_mut(&tid)
    }

    /// Returns a mutable reference to the currently running thread.
    fn current_mut(&mut self) -> Option<&mut Thread> {
        self.scheduler
            .current()
            .and_then(move |tid| self.get_mut(tid))
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

        thread.set_state(ThreadState::Running);
        thread
    }

    /// Register a thread as having returned and save its return value; keep it
    /// around until it is joined and reaped.
    fn exit(&mut self, status: isize) {
        let curr_tid = self
            .scheduler
            .current()
            .expect("scheduler returned no current thread");

        self.scheduler.complete(curr_tid);

        let curr = self
            .threads
            .get_mut(&curr_tid)
            .unwrap_or_else(|| panic!("no such thread {}", curr_tid));

        info!("Exit thread {} with status {}", curr, status);

        curr.exit(status);
    }
}
