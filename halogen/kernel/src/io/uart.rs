use alloc::boxed::Box;

use halogen_common::mem::{Address, VirtualAddress};

use super::console::register_console;
use crate::mem::{
    io::{UART_BASE, UART_SIZE},
    paging::{map, Permissions},
};

const UART_IRQ: usize = 10;

const DATA_OFFSET: usize = 0;
const INT_ENABLE_OFFSET: usize = 1;
const FIFO_OFFSET: usize = 2;
const LINE_CTL_OFFSET: usize = 3;
const MODEM_CTL_OFFSET: usize = 4;
const LINE_STAT_OFFSET: usize = 5;
const MODEM_STAT_OFFSET: usize = 6;
const SCRATCH_OFFSET: usize = 7;

/// Register the UART device as the main console.
pub fn use_as_console() {
    unsafe {
        register_console(Box::new(Ns16550aUart::new(
            map(None, Some(UART_BASE), UART_SIZE, Permissions::ReadWrite).unwrap(),
        )));
    }
}

/// Driver for the NS16550A UART device.
#[derive(Clone, Copy)]
pub struct Ns16550aUart {
    base: VirtualAddress,
}

impl Ns16550aUart {
    /// Create a new NS16550A UART module driver.
    ///
    /// # Safety
    ///
    /// - `base` must be mapped to a NS16550A device.
    pub const unsafe fn new(base: VirtualAddress) -> Ns16550aUart {
        Ns16550aUart { base }
    }

    /// Initialize the UART module registers.
    pub fn init(&mut self) {
        unsafe {
            ((self.base + FIFO_OFFSET).as_mut_ptr() as *mut u8).write_volatile(0b1);
            ((self.base + LINE_CTL_OFFSET).as_mut_ptr() as *mut u8).write_volatile(0b11);
            ((self.base + INT_ENABLE_OFFSET).as_mut_ptr() as *mut u8).write_volatile(0b1);
        }
    }
}

impl core::fmt::Write for Ns16550aUart {
    fn write_str(&mut self, str: &str) -> core::fmt::Result {
        for b in str.bytes() {
            unsafe {
                ((self.base + DATA_OFFSET).as_mut_ptr() as *mut u8).write_volatile(b);
            }
        }
        Ok(())
    }
}
