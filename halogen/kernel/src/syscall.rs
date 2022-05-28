use crate::{error::KernelError, log::*, task};

#[derive(Clone, Copy, Debug)]
pub enum Function {
    Yield,
    Invalid,
}

pub type SysCallResult = Result<Option<isize>, KernelError>;

impl From<usize> for Function {
    fn from(n: usize) -> Function {
        match n {
            1 => Function::Yield,
            _ => Function::Invalid,
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
    let func = Function::from(id);
    match match func {
        Function::Yield => syscall_yld(),
        Function::Invalid => Err(KernelError::InvalidSysCall),
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
pub fn syscall_yld() -> SysCallResult {
    task::yld();
    Ok(None)
}
