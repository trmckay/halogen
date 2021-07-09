use crate::{mmio_rd, mmio_wr};
use core::fmt::{Error, Write};

use super::DEV_UART0;

/// Memory address of the QEMU UART device.
const WRITE_OFFST: usize = 0x00;
const READ_OFFST: usize = 0x04;

const TX_CTRL_OFFST: usize = 0x08;
const RX_CTRL_OFFST: usize = 0x0C;

/// Driver for the UART module in the QEMU virt machine.
pub struct Uart {
    addr: usize,
}

impl Uart {
    pub fn new() -> Self {
        Uart { addr: DEV_UART0 }
    }

    pub fn read_byte(&self) -> u8 {
        unsafe {
            return mmio_rd!(self.addr + READ_OFFST);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            mmio_wr!(self.addr + WRITE_OFFST, byte);
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
