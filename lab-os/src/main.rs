#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    global_asm,
    asm,
    exclusive_range_pattern,
    custom_test_frameworks
)]
#![allow(dead_code)]
#![test_runner(test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use lazy_static::lazy_static;
use spin::Mutex;

#[cfg(not(test))]
use mem::page::PageAllocator;

mod boot;
mod cpu;
mod debug;
mod driver;
mod mem;
mod panic;
mod trap;
mod util;

#[cfg(not(test))]
lazy_static! {
    // Initialize a UART writer behind a spinlock mutex.
    pub static ref UART: Mutex<driver::uart::UartWriter> = Mutex::new(
        driver::uart::UartWriter::new(driver::uart::DEV_UART0)
    );


    #[cfg(ll_alloc)]
    pub static ref PAGE_ALLOCATOR: mem::page::ll_alloc::LinkedListAllocator = mem::page::ll_alloc::LinkedListAllocator;
}

/// Entry-point for the kernel. After the assembly-based set-up
/// is complete, the system will jump here.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    PAGE_ALLOCATOR.init();

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

    panic!();
}

#[cfg(test)]
mod test;

#[cfg(test)]
lazy_static! {
    // Initialize a UART writer behind a spinlock mutex.
    pub static ref UART: Mutex<driver::uart::UartWriter> = Mutex::new(
        driver::uart::UartWriter::new(driver::uart::DEV_UART0)
    );
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    test_main();
    loop {}
}
