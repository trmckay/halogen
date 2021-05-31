use crate::{mmio_rd, mmio_wr};
use core::fmt::{Error, Write};

// Device driver for NS16550A UART module provided by QEMU.
#[cfg(machine = "qemu")]
pub const DEFAULT_UART_MMIO_ADDR: usize = 0x10000000;

pub struct Uart {
    mmio_addr: usize,
}

impl Uart {
    pub fn new() -> Uart {
        Uart {
            mmio_addr: DEFAULT_UART_MMIO_ADDR,
        }
    }

    pub fn from_addr(mmio_addr: usize) -> Uart {
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

    pub fn read_byte(&self) -> u8 {
        unsafe {
            return mmio_rd!(self.mmio_addr);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            mmio_wr!(self.mmio_addr, byte);
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.write_byte(c);
        }
        Ok(())
    }
}

impl Default for Uart {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!(crate::driver::uart::Uart::new(), $($args)+
        );
    });
}

#[macro_export]
macro_rules! println
{
    () => ({
        print!("\n")
    });
    ($fmt:expr) => ({
        print!(concat!($fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\n"), $($args)+)
    });
}
