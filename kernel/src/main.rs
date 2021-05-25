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
    println!("Initialize kernel.\n");

    let mut uart = uart::Uart::new(uart::UART_MMIO_ADDR);
    uart.init();

    // Nothing crazy here, just a read-print loop.
    loop {
        let c = uart.read_byte();
        match c {
            // Null
            0x00 => (),
            // C-c || C-d
            0x03 | 0x04 => panic!(),
            // Return
            0x0D => println!(),
            // Backspace
            0x7F => print!("{} {}", 0x08 as char, 0x08 as char),
            // Everything else
            _ => print!("{}", c as char),
        }
    }
}
