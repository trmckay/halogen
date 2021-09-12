#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, asm, exclusive_range_pattern)]
#![allow(dead_code)]

use lazy_static::lazy_static;
use spin::Mutex;

mod boot;
mod cpu;
mod debug;
mod driver;
mod mem;
mod panic;
mod trap;
mod util;

lazy_static! {
    // Initialize a UART writer behind a spinlock mutex.
    pub static ref UART: Mutex<driver::uart::UartWriter> = Mutex::new(driver::uart::UartWriter::new(driver::uart::DEV_UART0));
}

/// Entry-point for the kernel. After the assembly-based set-up
/// is complete, the system will jump here.
#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    mem::pages::heap_init();

    println!("\nlab-os kernel v0.1.0");
    for _ in 0..64 {
        print!("=");
    }
    println!();

    println!(
        "Text:  {:p}..{:p} ({:5} KB)",
        text_begin!(),
        text_end!(),
        text_size!() / 1024
    );
    println!(
        "Stack: {:p}..{:p} ({:5} KB)",
        k_stack_begin!(),
        k_stack_end!(),
        k_stack_size!() / 1024
    );
    println!(
        "Heap:  {:p}..{:p} ({:5} KB)",
        k_heap_begin!(),
        k_heap_end!(),
        k_heap_size!() / 1024
    );

    mem::pages::heap_dump();

    panic!();
}
