use crate::{mmio_rd, mmio_wr};
use core::fmt::{Error, Write};

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
    /// pub const DEV_UART: usize = 0x1000;
    /// let mut uart = Uart(DEV_UART);
    /// ```
    pub fn new(phys_addr: usize) -> UartDriver {
        UartDriver { phys_addr }
    }

    pub fn init(&mut self) {
        mmio_wr!(self.phys_addr, 3, 0b11);
        mmio_wr!(self.phys_addr, 2, 0b1);
        mmio_wr!(self.phys_addr, 1, 0b1);
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
        return mmio_rd!(self.phys_addr);
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
        mmio_wr!(self.phys_addr, byte as u8);
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
    ($d:expr, $($args:tt)+) => ({
        use core::fmt::Write;

        let _ = write!($d, $($args)+);
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
    ($d:expr) =>  ({
        print!($d, "\n")
    });
    ($d:expr, $fmt:expr) => ({
        print!($d, concat!($fmt, "\n"))
    });
    ($d:expr, $fmt:expr, $($args:tt)+) => ({
        print!($d, concat!($fmt, "\n"), $($args)+)
    });
}

/// Read a single character from the primary UART device.
///
/// Example:
///
/// ```
/// ```
#[macro_export]
macro_rules! read_char {
    () => {
        (crate::driver::UartDriver::new(crate::driver::DEV_UART).read_byte() as char)
    };
}
