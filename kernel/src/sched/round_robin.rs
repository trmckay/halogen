use super::JobScheduler;
use crate::prelude::*;

#[derive(Default, Clone)]
pub struct RoundRobinScheduler {
    queue: VecDeque<usize>,
    tid: usize,
    current: Option<usize>,
}

impl JobScheduler for RoundRobinScheduler {
    type Handle = usize;

    fn add_new_with_priority(&mut self, _: isize) -> Option<Self::Handle> {
        let tid = self.tid;
        self.tid += 1;
        self.queue.push_back(tid);
        Some(tid)
    }

    fn set_priority(&self, _: Self::Handle, _: isize) {
        warn!("Priority has no effect in a round-robin scheduler",)
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
