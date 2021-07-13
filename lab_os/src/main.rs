#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, asm, exclusive_range_pattern)]
#![allow(dead_code)]

/// Delivers functionality related to debugging and error-reporting.
mod debug;

/// Contains device drivers and platform-specific code.
mod driver;

/// Implements the `panic` language feature.
mod panic;

/// Contains code run right after boot.
mod boot;

/// Entry-point for the kernel. After the assembly-based set-up
/// is complete, the system will jump here.
#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    let mut uart = driver::UartDriver::new(driver::DEV_UART);
    uart.init();

    loop {
        let c = read_char!();
        if c == '!' {
            panic!();
        } else if c != '\0' {
            print!(uart, "{}", c);
        }
    }
}

/// CPU trap-handler. When the CPU issues a trap, it will jump
/// here.
#[no_mangle]
pub extern "C" fn mtrap_vector() {
    unsafe {
        asm!("mret");
    }
}
