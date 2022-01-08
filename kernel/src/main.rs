#![no_std]
#![no_main]
#![feature(panic_info_message, exclusive_range_pattern, custom_test_frameworks)]
#![allow(arithmetic_overflow)]
#![allow(dead_code)]
#![test_runner(test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use lazy_static::lazy_static;
use spin::Mutex;

/// ANSI escape codes
mod ansi;
/// Bitwise manipulation utilities
mod bitwise;
/// Code run at boot
mod boot;
/// Debug utilities
mod debug;
/// Device drivers
mod driver;
/// Memory and MMU
mod mem;
/// Panic language-feature
mod panic;
/// Trap vectors
mod trap;

#[cfg(not(test))]
lazy_static! {
    /// Singleton `UART` structure for printing to the console.
    pub static ref UART: Mutex<driver::uart::UartWriter> =
        Mutex::new(driver::uart::UartWriter::new(driver::uart::DEV_UART));

    /// Singleton `QEMU_EXIT` to allow interacting with the host QEMU process.
    pub static ref QEMU_EXIT: qemu_exit::RISCV64 = qemu_exit::RISCV64::new(driver::DEV_TEST as u64);
}

/// Exit the QEMU virtual machine and return exit code 1 to the calling shell.
#[macro_export]
macro_rules! exit_failure {
    () => {
        use qemu_exit::QEMUExit;
        crate::QEMU_EXIT.exit_failure();
    };
}

/// Exit the QEMU virtual machine and return exit code 0 to the calling shell.
#[macro_export]
macro_rules! exit_success {
    () => {
        use qemu_exit::QEMUExit;
        crate::QEMU_EXIT.exit_success();
    };
}

#[cfg(not(test))]

/// Entry-point for the kernel. After the assembly-based set-up
/// is complete, the system will jump here
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    println!(
        "\n{}halogen kernel v0{}",
        ansi::Color::Cyan,
        ansi::Color::Reset
    );

    sv39_enable!();

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
