#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, asm, exclusive_range_pattern)]
#![allow(dead_code)]

global_asm!(include_str!("boot/boot.s"));

pub mod driver;
pub mod memory;
pub mod process;

mod panic;

#[no_mangle]
pub extern "C" fn kernel() -> ! {
    println!("Hello, world.");
    panic!();
}

#[no_mangle]
pub extern "C" fn mtrap_vector() {
    unsafe {
        asm!("mret");
    }
}

#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
        // Use the platform-specific UART driver.
        use crate::driver::Uart;
        use core::fmt::Write;

        let mut uart = Uart::new();
        let _ = write!(uart, $($args)+);
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
