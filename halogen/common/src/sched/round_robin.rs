#[cfg(not(test))]
use alloc::collections::VecDeque;
#[cfg(test)]
use std::collections::VecDeque;

use super::TaskScheduler;

/// Round-robin task scheduler.
#[derive(Default, Clone)]
pub struct RoundRobinScheduler {
    queue: VecDeque<usize>,
    current: Option<usize>,
}

impl TaskScheduler for RoundRobinScheduler {
    type Handle = usize;

    fn add_with_priority(&mut self, id: Self::Handle, _priority: isize) {
        self.queue.push_back(id);
    }

    fn set_priority(&self, _: Self::Handle, _: isize) {
        unimplemented!()
    }

    fn next(&mut self) -> Option<Self::Handle> {
        if let Some(job) = self.current {
            self.queue.push_back(job);
        }

        let next = self.queue.pop_front();
        self.current = next;

        next
    }

    fn complete(&mut self, job: Self::Handle) {
        self.current = None;
        self.queue.retain(|&j| j != job);
    }

    fn current(&self) -> Option<Self::Handle> {
        self.current
    }

    fn yld(&mut self, job: Self::Handle) {
        self.queue.retain(|&j| j != job);
        self.queue.push_back(job);
    }
}
