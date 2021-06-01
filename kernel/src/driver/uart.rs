use crate::{mmio_rd, mmio_wr};
pub use core::fmt::{Error, Write};

pub trait UartDriver: core::fmt::Write {
    fn new() -> Self;
    fn init(&mut self);
    fn read_byte(&self) -> u8;
    fn write_byte(&mut self, byte: u8);
}

pub struct UartQemu {
    addr: usize,
}

pub const QEMU_UART_ADDR: usize = 0x10000000;

impl UartDriver for UartQemu {
    fn new() -> Self {
        UartQemu {
            addr: QEMU_UART_ADDR,
        }
    }

    fn init(&mut self) {
        let mmio_ptr = self.addr as *mut u8;
        unsafe {
            mmio_ptr.add(3).write_volatile(0b11);
            mmio_ptr.add(2).write_volatile(0b1);
            mmio_ptr.add(1).write_volatile(0b1);
        }
    }

    fn read_byte(&self) -> u8 {
        unsafe {
            return mmio_rd!(self.addr);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        unsafe {
            mmio_wr!(self.addr, byte);
        }
    }
}

impl Write for UartQemu {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.write_byte(c);
        }
        Ok(())
    }
}
