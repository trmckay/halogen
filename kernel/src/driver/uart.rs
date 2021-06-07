use crate::{mmio_rd, mmio_wr};
use core::fmt::{Error, Write};

pub trait UartDriver {
    fn new() -> Self;

    fn read_byte(&self) -> u8;

    fn write_byte(&mut self, byte: u8);
}

pub const QEMU_UART_ADDR: usize = 0x10000000;

pub struct UartQemu {
    addr: usize,
}

impl UartDriver for UartQemu {
    fn new() -> Self {
        let uart = UartQemu {
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
        for b in s.bytes() {
            self.write_byte(b);
        }
        Ok(())
    }
}
