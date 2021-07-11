// Platform-specific drivers should be collected in a
// subdirectory of `driver`, i.e. `driver/virt`.

// If including a module tree for a platform and its
// drivers, the module must export a function
// `platform_init() -> ()` which initializes any
// hardware that the platform needs.

/// This module contains platform-specific drivers
/// for the QEMU virt machine.
mod virt;
pub use virt::*;

/// Write to an MMIO address.
///
/// Examples:
///
/// ```
/// // Write 17 to address 0x1000.
/// mmio_wr!(0x1000, 17)
///
/// // Write 17 to address 0x1004.
/// mmio_wr!(0x1000, 0x4, 17)
/// ```
#[macro_export]
macro_rules! mmio_wr {
    ($a:expr, $d:expr) => {
        unsafe {
            ($a as *mut u8).write_volatile($d);
        }
    };
    ($a:expr, $o:expr, $d:expr) => {
        unsafe {
            ($a as *mut u8).add($o).write_volatile($d);
        }
    };
}

/// Read from an MMIO address.
///
/// Examples:
///
/// ```
/// // Read from address 0x1000.
/// mmio_wr!(0x1000, 17)
///
/// // Read from address 0x1004.
/// mmio_wr!(0x1000, 0x4, 17)
/// ```
#[macro_export]
macro_rules! mmio_rd {
    ($a:expr) => {
        unsafe { ($a as *mut u8).read_volatile() }
    };
    ($a:expr, $o:expr) => {
        unsafe { ($a as *mut u8).add($o).read_volatile() }
    };
}

/// Read a character from the UART input.
///
/// Example:
///
/// ```
/// let c = get_char!();
/// assert!(c == 't');
/// ```
#[macro_export]
macro_rules! get_char {
    () => {
        (crate::driver::Uart.read_byte() as char)
    };
}

/// Output formatted text to the UART device.
#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
        use core::fmt::Write;
        write!(crate::driver::Uart, $($args)+).unwrap()
    });
}

/// Like `print!`, but adds a newline to the end of the stirng.
#[macro_export]
macro_rules! println
{
    () => ({
        print!("\n")
    });
    ($fmt:expr) => ({
        print!(concat!($fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\n"), $($args)+)
    });
}
