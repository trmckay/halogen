#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, asm, exclusive_range_pattern)]
#![allow(dead_code)]

/// Delivers functionality related to debugging and
/// error-reporting.
mod debug;

/// Contains device drivers and platform-specific
/// code.
mod driver;

/// Implements the `panic` language feature.
mod panic;

mod boot;

/// Entry-point for the kernel. After the assembly-based set-up
/// is complete, the system will jump here.
#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    // Initialize any hardware that needs to be
    // set up before the kernel runs.
    driver::platform_init();

    loop {
        let c = get_char!();
        if c != 'p' {
            print!("{}", c);
        } else {
            panic!();
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
