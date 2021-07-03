#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, asm, exclusive_range_pattern)]

global_asm!(include_str!("boot/boot.s"));
global_asm!(include_str!("boot/trap.s"));

pub mod driver;
pub mod memory;
pub mod process;

mod panic;
mod print;

#[no_mangle]
pub extern "C" fn kernel() -> ! {
    println!("Hello, world.");
    panic!();
}
