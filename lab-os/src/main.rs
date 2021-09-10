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

/// General utility macros and functions.
mod util;

/// Drivers relating to memory.
mod mem;

/// Entry-point for the kernel. After the assembly-based set-up
/// is complete, the system will jump here.
#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    // Init NS16550 UART device.
    if cfg!(machine = "virt") {
        phys_write!(driver::uart::DEV_UART0, 3, 0b11);
        phys_write!(driver::uart::DEV_UART0, 2, 0b1);
        phys_write!(driver::uart::DEV_UART0, 1, 0b1);
    }

    println!("\nlab-os kernel v0.1.0");
    for _ in 0..64 {
        print!("=");
    }
    println!();

    println!(
        "Text:  {:p}..{:p} ({:5} KB)",
        text_begin!(),
        text_end!(),
        text_size!() / (8 * 1024)
    );
    println!(
        "Stack: {:p}..{:p} ({:5} KB)",
        k_stack_begin!(),
        k_stack_end!(),
        k_stack_size!() / (8 * 1024)
    );
    println!(
        "Heap:  {:p}..{:p} ({:5} KB)",
        k_heap_begin!(),
        k_heap_end!(),
        k_heap_size!() / (8 * 1024)
    );

    mem::pages::init_heap();

    println!();
    mem::pages::dump_free_blocks();

    loop {}
}

/// CPU trap-handler. When the CPU issues a trap, it will jump
/// here.
#[no_mangle]
pub extern "C" fn mtrap_vector() {
    unsafe {
        asm!("mret");
    }
}
