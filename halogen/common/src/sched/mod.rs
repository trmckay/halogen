/// FIFO scheduler.
mod fifo;
/// Round-robin scheduler.
mod round_robin;
/// Scheduler interface.
mod scheduler;

pub use fifo::FifoScheduler;
pub use round_robin::RoundRobinScheduler;
pub use scheduler::TaskScheduler;
