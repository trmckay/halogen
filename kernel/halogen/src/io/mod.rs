pub mod char;
pub mod uart;

#[derive(Debug, Copy, Clone)]
pub enum DeviceError {
    Uninit,
    Read,
    Write,
}
