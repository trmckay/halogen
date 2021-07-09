use crate::{mmio_rd, mmio_wr};
use core::fmt::{Error, Write};

use super::DEV_UART;

/// Driver for the UART module in the QEMU virt machine.
pub struct Uart {
    addr: *mut u8,
}

impl Uart {
    pub fn new() -> Self {
        let uart = Uart {
            addr: DEV_UART as *mut u8,
        };
        unsafe {
            mmio_wr!(uart.addr, 3, 0b11);
            mmio_wr!(uart.addr, 2, 0b1);
            mmio_wr!(uart.addr, 1, 0b1);
        }
        uart
    }

    pub fn read_byte(&self) -> u8 {
        unsafe {
            return mmio_rd!(self.addr);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            mmio_wr!(self.addr, byte);
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for b in s.bytes() {
            self.write_byte(b);
        }
        Ok(())
    }
}
