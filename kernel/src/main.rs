#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, asm, exclusive_range_pattern)]

global_asm!(include_str!("boot/boot.s"));
global_asm!(include_str!("boot/trap.s"));

// Private modules.
mod panic;

// Public modules.
pub mod driver;
pub mod util;

#[no_mangle]
pub extern "C" fn kernel() -> ! {
    println!("Hello, world.");
    panic!();
}

#[cfg(machine = "qemu")]
#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
        use crate::driver::{UartDriver, UartQemu};
        use core::fmt::Write;
        let _ = write!(UartQemu::new(), $($args)+
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
