use core::arch::asm;

mod hsm;
mod timer;

pub use timer::set_timer;

#[derive(Clone, Copy, Debug)]
pub enum SbiError {
    Failed,
    NotSupported,
    InvalidParameter,
    Denied,
    InvalidAddress,
    AlreadyAvailable,
}

impl SbiError {
    pub fn from_isize(error: isize) -> Option<SbiError> {
        match error {
            -1 => Some(SbiError::Failed),
            -2 => Some(SbiError::NotSupported),
            -3 => Some(SbiError::InvalidParameter),
            -4 => Some(SbiError::Denied),
            -5 => Some(SbiError::InvalidAddress),
            -6 => Some(SbiError::AlreadyAvailable),
            _ => None,
        }
    }
}

fn sbi_ecall(args: [usize; 6], ext: usize, func: usize) -> Result<usize, SbiError> {
    let error: isize;
    let val: usize;

    unsafe {
        asm!(
            "ecall",
            in("a0") args[0],
            in("a1") args[1],
            inout("a2") args[2] => _,
            inout("a3") args[3] => _,
            inout("a4") args[4] => _,
            inout("a5") args[5] => _,
            inout("a6") func => _,
            inout("a7") ext => _,
            lateout("a0") error,
            lateout("a1") val,
        );
    }

    match SbiError::from_isize(error) {
        Some(e) => Err(e),
        None => Ok(val),
    }
}
