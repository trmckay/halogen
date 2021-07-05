use crate::{mmio_rd, mmio_wr};
use core::fmt::{Error, Write};

/// Memory address of the QEMU UART device.
pub const QEMU_UART_ADDR: usize = 0x10000000;

/// Driver for the UART module in the QEMU virt machine.
pub struct Uart {
    addr: usize,
}

impl Uart {
    pub fn new() -> Self {
        let uart = Uart {
            addr: QEMU_UART_ADDR,
        };
        let mmio_ptr = uart.addr as *mut u8;
        unsafe {
            mmio_ptr.add(3).write_volatile(0b11);
            mmio_ptr.add(2).write_volatile(0b1);
            mmio_ptr.add(1).write_volatile(0b1);
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
