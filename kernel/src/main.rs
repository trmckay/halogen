#![no_std]
#![no_main]

mod panic;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let str = b"Hello, world.";

    loop {}
}
