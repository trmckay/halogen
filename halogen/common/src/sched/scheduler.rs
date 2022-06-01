/// Interface for task schedulers. This is currently unfinished; the interface
/// will likely shift when more complicated schedulers are added.
pub trait TaskScheduler: Sync + Send {
    type Handle: Copy + Sized + Sync + Send + Eq;

    /// Add a task to the scheduling pool.
    fn add_with_priority(&mut self, id: Self::Handle, priority: isize);

    /// Return the next task and internally mark it as running.
    fn next(&mut self) -> Option<Self::Handle>;

    /// Complete a task and remove it from the pool.
    fn complete(&mut self, job: Self::Handle);

    /// Get the handle of the currently running task.
    fn current(&self) -> Option<Self::Handle>;

    /// Set the priority for a task.
    fn set_priority(&self, job: Self::Handle, priority: isize);

    /// Yield the currently running task's remaining time.
    fn yld(&mut self, job: Self::Handle);

    /// Add a new task with the lowest priority.
    fn add_new(&mut self, id: Self::Handle) {
        self.add_with_priority(id, 0);
    }
}
