#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, asm, exclusive_range_pattern)]
#![allow(dead_code)]

global_asm!(include_str!("boot/boot.s"));

pub mod driver;
pub mod memory;
pub mod process;

mod panic;

/// Entry-point for the kernel. After the assembly-based set-up
/// is complete, the system will jump here.
#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    println!("Hello, world.");
    panic!();
}

/// CPU trap-handler. When the CPU issues a trap, it will jump
/// here.
#[no_mangle]
pub extern "C" fn mtrap_vector() {
    unsafe {
        asm!("mret");
    }
}

/// `print!` and `println!` are used to output to the console. They
/// use a platform-specific UART driver as re-exported in `driver`.
#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
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
