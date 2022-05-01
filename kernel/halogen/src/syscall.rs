use crate::{prelude::*, thread};

#[derive(Clone, Copy, Debug)]
pub enum SyscallFunction {
    Yield,
    Invalid,
}

pub type SyscallResult = Result<Option<isize>, SyscallError>;

#[derive(Clone, Copy, Debug)]
pub enum SyscallError {
    Denied,
    InvalidId,
}

impl From<usize> for SyscallFunction {
    fn from(n: usize) -> SyscallFunction {
        match n {
            1 => SyscallFunction::Yield,
            _ => SyscallFunction::Invalid,
        }
    }
}

/// # Safety
///
/// Should only be called from the trap handler
pub unsafe extern "C" fn handle_syscall(
    id: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
) -> isize {
    let func = SyscallFunction::from(id);
    match match func {
        SyscallFunction::Yield => syscall_yld(),
        SyscallFunction::Invalid => Err(SyscallError::InvalidId),
    } {
        Ok(Some(val)) => val,
        Ok(None) => 0,
        Err(why) => {
            error!(
                "syscall error: {:?}({}, {}, {}, {}, {}): {:?}",
                func, a1, a2, a3, a4, a5, why
            );
            -1
        }
    }
}

/// Elect to block a thread as a syscall
pub fn syscall_yld() -> SyscallResult {
    thread::yld();
    Ok(None)
}
