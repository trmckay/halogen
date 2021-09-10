pub use core::fmt::{Error, Write};

use crate::{phys_read, phys_write};

#[cfg(platform = "virt")]
pub const DEV_UART0: usize = 0x1000_0000;

/// Simple handle for the UART device.
/// This is implemented as a struct so
/// we can use the Write trait.
///
/// We also have the advantage of enforcing
/// a mutable access on writes.
pub struct UartDriver {
    phys_addr: usize,
}

impl UartDriver {
    /// Read a single byte from the device.
    ///
    /// Example:
    ///
    /// ```
    /// pub const DEV_UART0: usize = 0x1000;
    /// let mut uart = Uart(DEV_UART0);
    /// ```
    pub fn new(phys_addr: usize) -> UartDriver {
        UartDriver { phys_addr }
    }

    /// Read a single byte from the device.
    ///
    /// Example:
    ///
    /// ```
    /// let c: u8 = uart.read_byte();
    /// ```
    #[inline]
    pub fn read_byte(&self) -> u8 {
        return phys_read!(self.phys_addr);
    }

    /// Write a single byte to the device.
    ///
    /// Example:
    ///
    /// ```
    /// let c: u8 = 0x17;
    /// uart.write_byte(c);
    /// ```
    #[inline]
    pub fn write_byte(&mut self, byte: u8) {
        phys_write!(self.phys_addr, byte as u8);
    }
}

// Implement the `Write` trait so we can
// print format strings.
impl Write for UartDriver {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for b in s.bytes() {
            self.write_byte(b);
        }
        Ok(())
    }
}

/// Print a format string to the UART device.
///
/// Example:
///
/// ```
/// ```
#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
        use crate::driver::uart::{UartDriver, DEV_UART0};
        use core::fmt::Write;
        let _ = write!(UartDriver::new(DEV_UART0), $($args)+);
    });
}

/// Print a format string to the UART device with a trailing newline.
///
/// Example:
///
/// ```
/// ```
#[macro_export]
macro_rules! println
{
    () =>  ({
        print!("\n")
    });
    ($fmt:expr) => ({
        print!(concat!($fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\n"), $($args)+)
    });
}
