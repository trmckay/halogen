use core::arch::asm;

pub mod uart;

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

pub fn sbi_ecall(args: [usize; 6], func: usize, ext: usize) -> Result<usize, SbiError> {
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

pub const UART_EXT_ID: usize = 0;
pub const UART_PUTC_FUNC_ID: usize = 1;
pub const UART_GETC_FUNC_ID: usize = 2;
