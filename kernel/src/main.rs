#![no_std] // Disable the standard library.
#![no_main] // We are going to define the entrypoint manually.
#![feature(panic_info_message, global_asm, asm)]

global_asm!(include_str!("boot/boot.s"));
global_asm!(include_str!("boot/trap.s"));

pub mod mmio;
pub mod panic;
pub mod uart;

#[no_mangle]
pub extern "C" fn kernel() {
    let mut uart = uart::Uart::new(uart::UART_MMIO_ADDR);
    uart.init();

    let str = b"Hello, world.\n";

    for c in str.iter() {
        uart.write(*c);
    }

    loop {}
}
