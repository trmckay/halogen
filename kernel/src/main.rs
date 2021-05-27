#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, asm, exclusive_range_pattern)]

global_asm!(include_str!("boot/boot.s"));
global_asm!(include_str!("boot/trap.s"));

// Private modules.
mod panic;

// Public modules.
pub mod dump;
pub mod mmio;
pub mod uart;

#[no_mangle]
pub extern "C" fn kernel() -> ! {
    println!("rVr-kernel");
    panic!();
}
