pub trait JobScheduler: Sync + Send {
    type Handle: Copy + Sized + Sync + Send;

    fn add_new_with_priority(&mut self, priority: isize) -> Option<Self::Handle>;
    fn next(&mut self) -> Option<Self::Handle>;
    fn complete(&mut self, job: Self::Handle);
    fn current(&self) -> Option<Self::Handle>;
    fn set_priority(&self, job: Self::Handle, priority: isize);
    fn yld(&mut self, job: Self::Handle);

    fn add_new(&mut self) -> Option<Self::Handle> {
        self.add_new_with_priority(0)
    }
}
