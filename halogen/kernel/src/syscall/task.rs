use crate::task::{executor::pid, exit};

pub(super) fn syscall_exit(status: isize) -> isize {
    exit(status);
    0
}

pub(super) fn syscall_pid() -> isize {
    pid().expect("caller has no PID") as isize
}
