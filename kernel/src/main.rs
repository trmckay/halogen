#![no_std] // Disable the standard library.
#![no_main] // We are going to define the entrypoint manually.
#![feature(panic_info_message, global_asm, asm)]

global_asm!(include_str!("boot/boot.s"));
global_asm!(include_str!("boot/trap.s"));

pub mod mmio;
pub mod panic;
pub mod uart;

#[macro_export]
macro_rules! print
{
	($($args:tt)+) => ({
			use core::fmt::Write;
			let _ = write!(crate::uart::Uart::new(uart::UART_MMIO_ADDR), $($args)+);
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

#[no_mangle]
pub extern "C" fn kernel() {
    println!("Hello, world.");
    loop {}
}
