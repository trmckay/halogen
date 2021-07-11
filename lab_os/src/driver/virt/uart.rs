use crate::{mmio_rd, mmio_wr};
use core::fmt::{Error, Write};

/// Physical address of the UART device.
pub const DEV_UART: usize = 0x10000000;

/// Simple handle for the UART device.
/// This is implemented as a struct so
/// we can use the Write trait.
///
/// We also have the advantage of enforcing
/// a mutable access on writes.
pub struct Uart;

impl Uart {
    /// Initial setup of the device.
    pub fn init() {
        mmio_wr!(DEV_UART, 3, 0b11);
        mmio_wr!(DEV_UART, 2, 0b1);
        mmio_wr!(DEV_UART, 1, 0b1);
    }

    /// Read a single byte from the device.
    #[inline]
    pub fn read_byte(&self) -> u8 {
        return mmio_rd!(DEV_UART);
    }

    /// Write a single byte to the device.
    #[inline]
    pub fn write_byte(&mut self, byte: u8) {
        mmio_wr!(DEV_UART, byte);
    }
}

// Implement the `Write` trait so we can
// print format strings.
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for b in s.bytes() {
            self.write_byte(b);
        }
        Ok(())
    }
}
