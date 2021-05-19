#![no_std]
#![no_main]

mod mmio;
mod panic;

use core::ptr::*;
use mmio::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {
        unsafe {
            let v = read_volatile(mmio::SWITCHES);
            write_volatile(mmio::LEDS, v);
            write_volatile(mmio::SSEG, 0xFFFF - v);
        }
    }
}
