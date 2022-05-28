use alloc::boxed::Box;
use core::fmt::Write;

use spin::Mutex;

use crate::{log::*, sbi::console::SbiConsole};

static mut FIRMWARE_CONSOLE: SbiConsole = SbiConsole;

static mut CONSOLE: Option<Mutex<Box<dyn Write>>> = None;

pub fn register_console(console: Box<dyn Write>) {
    info!("Register new console");
    unsafe { CONSOLE = Some(Mutex::new(console)) }
}

/// Print using the lock-protected kernel API
pub fn kprint(args: core::fmt::Arguments) -> Result<(), core::fmt::Error> {
    unsafe {
        match &CONSOLE {
            Some(console) => console.lock().write_fmt(args),
            None => fwprint(args),
        }
    }
}

/// Print using the firmware API
pub fn fwprint(args: core::fmt::Arguments) -> Result<(), core::fmt::Error> {
    unsafe { FIRMWARE_CONSOLE.write_fmt(args) }
}

/// Print a static string using the SBI firmware console
pub fn early_print(str: &str) {
    unsafe { FIRMWARE_CONSOLE.write_str(str).unwrap() };
}

/// Print a static string with a newline using the SBI firmware console
pub fn early_println(str: &str) {
    early_print(str);
    early_print("\n");
}

/// Kernel API for printing
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::io::console::kprint(format_args!($($arg)*)).unwrap());
}

/// Kernel API for printing
#[macro_export]
macro_rules! kprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

/// Firmware API for  printing
#[macro_export]
macro_rules! fwprint {
    ($($arg:tt)*) => ($crate::io::console::fwprint(format_args!($($arg)*)).unwrap());
}

/// Firmware API for  printing
#[macro_export]
macro_rules! fwprintln {
    () => ($crate::print_unsafe!("\n"));
    ($($arg:tt)*) => ($crate::fwprint!("{}\n", format_args!($($arg)*)));
}
