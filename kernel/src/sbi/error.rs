#[derive(Clone, Copy, Debug)]
pub enum SbiError {
    Failed,
    NotSupported,
    InvalidParameter,
    Denied,
    InvalidAddress,
    AlreadyAvailable,
}

impl From<isize> for SbiError {
    fn from(i: isize) -> SbiError {
        match i {
            -1 => SbiError::Failed,
            -2 => SbiError::NotSupported,
            -3 => SbiError::InvalidParameter,
            -4 => SbiError::Denied,
            -5 => SbiError::InvalidAddress,
            -6 => SbiError::AlreadyAvailable,
            _ => panic!("unknown SBI error type"),
        }
    }
}
