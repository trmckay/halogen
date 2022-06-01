use crate::task::exit;

pub(super) fn syscall_exit(status: isize) -> isize {
    exit(status);
    0
}
