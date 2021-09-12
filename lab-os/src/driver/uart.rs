pub use core::fmt::{Arguments, Error, Write};

use crate::{phys_read, phys_write};

#[cfg(platform = "virt")]
pub const DEV_UART0: usize = 0x1000_0000;

/// Simple handle for the UART device.
/// This is implemented as a struct so
/// we can use the Write trait.
///
/// We also have the advantage of enforcing
/// a mutable access on writes.
pub struct UartWriter {
    phys_addr: usize,
}

impl UartWriter {
    /// Read a single byte from the device.
    ///
    /// Example:
    ///
    /// ```
    /// pub const DEV_UART0: usize = 0x1000;
    /// let mut uart = Uart(DEV_UART0);
    /// ```
    pub fn new(phys_addr: usize) -> UartWriter {
        if cfg!(machine = "virt") {
            phys_write!(DEV_UART0, 3, 0b11);
            phys_write!(DEV_UART0, 2, 0b1);
            phys_write!(DEV_UART0, 1, 0b1);
        }
        UartWriter { phys_addr }
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
impl Write for UartWriter {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for b in s.bytes() {
            self.write_byte(b);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    #[allow(unused_imports)]
    use core::fmt::Write;
    crate::UART.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::driver::uart::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
