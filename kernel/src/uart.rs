use crate::{mmio_rd, mmio_wr};

// Device driver for NS16550A UART module provided by QEMU.
// See the specifications here: https://mth.st/blog/riscv-qemu/AN-491.pdf
//
pub const UART_MMIO_ADDR: usize = 0x10000000;

pub struct Uart {
    mmio_addr: usize,
}

impl Uart {
    pub fn new(mmio_addr: usize) -> Uart {
        Uart { mmio_addr }
    }

    pub fn init(&mut self) {
        let mmio_ptr = self.mmio_addr as *mut u8;
        unsafe {
            mmio_ptr.add(3).write_volatile(0b11);
            mmio_ptr.add(2).write_volatile(0b1);
            mmio_ptr.add(1).write_volatile(0b1);
        }
    }

    pub fn read(&self) -> u8 {
        unsafe {
            return mmio_rd!(self.mmio_addr);
        }
    }

    pub fn write(&mut self, byte: u8) {
        unsafe {
            mmio_wr!(self.mmio_addr, byte);
        }
    }
}
