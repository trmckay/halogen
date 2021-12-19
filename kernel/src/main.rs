#![no_std]
#![no_main]
#![feature(panic_info_message, exclusive_range_pattern, custom_test_frameworks)]
#![allow(dead_code)]
#![test_runner(test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use lazy_static::lazy_static;
use spin::Mutex;

mod ansi;
mod boot;
mod debug;
mod driver;
mod mem;
mod panic;
mod trap;
mod util;

#[cfg(not(test))]
lazy_static! {
    pub static ref UART: Mutex<driver::uart::UartWriter> =
        Mutex::new(driver::uart::UartWriter::new(driver::uart::DEV_UART));
    pub static ref QEMU_EXIT: qemu_exit::RISCV64 = qemu_exit::RISCV64::new(driver::DEV_TEST as u64);
}

/// Entry-point for the kernel. After the assembly-based set-up
/// is complete, the system will jump here.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    println!(
        "\n{}halogen kernel v0{}",
        ansi::Color::Cyan,
        ansi::Color::Reset
    );
    for _ in 0..48 {
        print!("=");
    }
    println!();

    println!(
        "Text:  {:p}..{:p} ({:6} KB)",
        text_begin!(),
        text_end!(),
        text_size!() / 1024
    );
    println!(
        "Stack: {:p}..{:p} ({:6} KB)",
        k_stack_begin!(),
        k_stack_end!(),
        k_stack_size!() / 1024
    );
    println!(
        "Heap:  {:p}..{:p} ({:6} KB)",
        k_heap_begin!(),
        k_heap_end!(),
        k_heap_size!() / 1024
    );

    panic!("end of kernel_start");
}

#[cfg(test)]
mod test;

#[cfg(test)]
lazy_static! {
    pub static ref UART: Mutex<driver::uart::UartWriter> =
        Mutex::new(driver::uart::UartWriter::new(driver::uart::DEV_UART));
    pub static ref QEMU_EXIT: qemu_exit::RISCV64 = qemu_exit::RISCV64::new(driver::DEV_TEST as u64);
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    test_main();
    exit_success!();
}
