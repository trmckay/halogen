#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, asm, exclusive_range_pattern)]

global_asm!(include_str!("boot/boot.s"));
global_asm!(include_str!("boot/trap.s"));

mod panic;
mod print;

pub mod driver;
pub mod util;

#[no_mangle]
pub extern "C" fn kernel() -> ! {
    println!("Hello, world.");
    panic!();
}
