use core::{fmt, fmt::Write};

use halogen_derive::ByteConsumerWrite;

use crate::{io, io::char::*, irq::plic, mem, prelude::*};

const UART_IRQ: u32 = 10;

const DATA_OFFSET: usize = 0;
const INT_ENABLE_OFFSET: usize = 1;
const FIFO_OFFSET: usize = 2;
const LINE_CTL_OFFSET: usize = 3;
const MODEM_CTL_OFFSET: usize = 4;
const LINE_STAT_OFFSET: usize = 5;
const MODEM_STAT_OFFSET: usize = 6;
const SCRATCH_OFFSET: usize = 7;

const QUEUE_SIZE: usize = 8;

lazy_static! {
    /// Mutex-protected reference to the Ns16550a UART module
    ///
    /// This only exists to provide some ordering guarantees for printing; do not use in interrupts
    pub static ref UART: Mutex<Ns16550a> =
        Mutex::new(unsafe { Ns16550a::new(mem::DEV_UART + mem::MMIO_OFFSET) });
}

#[derive(ByteConsumerWrite)]
pub struct Ns16550a {
    base: usize,
}

impl Ns16550a {
    /// Create a new NS16550A UART module driver
    ///
    /// # Safety
    ///
    /// * `base` must be mapped to a NS16550A device
    pub unsafe fn new(base: usize) -> Ns16550a {
        Ns16550a { base }
    }

    /// Initialize the UART module registers
    pub fn init(&mut self) {
        unsafe {
            ((self.base + FIFO_OFFSET) as *mut u8).write_volatile(0b1);
            ((self.base + LINE_CTL_OFFSET) as *mut u8).write_volatile(0b11);
            ((self.base + INT_ENABLE_OFFSET) as *mut u8).write_volatile(0b1);
        }
    }
}

impl io::char::ByteProducer for Ns16550a {
    fn read_byte(&mut self) -> Result<Option<u8>, io::DeviceError> {
        Ok(Some(unsafe {
            ((self.base + DATA_OFFSET) as *const u8).read_volatile()
        }))
    }
}

impl io::char::ByteConsumer for Ns16550a {
    fn write_byte(&mut self, byte: u8) -> Result<(), io::DeviceError> {
        unsafe { ((self.base + DATA_OFFSET) as *mut u8).write_volatile(byte) };
        Ok(())
    }
}

/// Print to the UART module
///
/// Do not call directly; use `print!()` and `println!()` instead
pub fn print_debug(args: fmt::Arguments) -> Result<(), fmt::Error> {
    let mut uart = UART.lock();
    uart.write_fmt(args)?;
    Ok(())
}

/// Print without locking
///
/// # Safety
///
/// Only use when absolutely necessary in interrupt handlers
pub unsafe fn print_debug_no_lock(args: fmt::Arguments) -> Result<(), fmt::Error> {
    let mut uart = Ns16550a::new(mem::DEV_UART + mem::MMIO_OFFSET);
    uart.write_fmt(args)?;
    Ok(())
}

/// Print a string over UART
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::uart::print_debug(format_args!($($arg)*)).unwrap());
}

/// Print a string and newline over UART
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Print a string over UART without locking
#[macro_export]
macro_rules! print_unsafe {
    ($($arg:tt)*) => ($crate::io::uart::print_debug_no_lock(format_args!($($arg)*)).unwrap());
}

/// Print a string and newline over UART without locking
#[macro_export]
macro_rules! println_unsafe {
    () => ($crate::print_unsafe!("\n"));
    ($($arg:tt)*) => ($crate::print_unsafe!("{}\n", format_args!($($arg)*)));
}

/// Echo a byte back to the console
fn echo_byte() {
    let mut uart = unsafe { Ns16550a::new(mem::DEV_UART + mem::MMIO_OFFSET) };
    if let Ok(Some(b)) = uart.read_byte() {
        unsafe { print_unsafe!("{}", b as char) };
    }
}

/// Trigger an external interrupt from the PLIC when the UART has data
pub fn enable_plic_int() {
    plic::register_isr(UART_IRQ, echo_byte);
    plic::enable(UART_IRQ);
    plic::set_priority(UART_IRQ, 7);
}
