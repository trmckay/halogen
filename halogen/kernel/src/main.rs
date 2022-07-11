#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    exclusive_range_pattern,
    const_maybe_uninit_zeroed,
    custom_test_frameworks,
    naked_functions,
    fn_align,
    asm_sym,
    asm_const,
    alloc_error_handler,
    stmt_expr_attributes,
    is_some_with,
    extern_types
)]
#![allow(arithmetic_overflow)]
#![allow(dead_code)]
#![test_runner(crate::tests::run_tests)]
#![reexport_test_harness_main = "test_harness"]

extern crate alloc;

#[cfg(not(target_arch = "riscv64"))]
core::compile_error!("rv64gc+sv39 is the only supported platform");

/// Unit and integration tests.
#[cfg(test)]
mod tests;

/// Entry-point for kernel.
mod boot;
/// Panic language-feature.
mod panic;

/// Architecture state and functionality.
pub mod arch;
/// Kernel error type.
pub mod error;
/// I/O devices.
pub mod io;
/// Interrupt request configuration.
pub mod irq;
/// Debug logging over UART.
pub mod log;
/// Memory management.
pub mod mem;
/// Interfacing with OpenSBI.
pub mod sbi;
/// System call definitions.
pub mod syscall;
/// Processes and threads.
pub mod task;
/// Trap handler.
pub mod trap;

const LOG_LEVEL: log::Level = log::Level::Trace;

/// Entry-point for the kernel.
///
/// # Safety
///
/// - This is only called once by the bootstrap code.
#[allow(named_asm_labels)]
#[repr(align(4))]
pub unsafe extern "C" fn kinit() -> ! {
    mem::heap::init();

    log::set_level(LOG_LEVEL);

    trap::init();

    // Until now, we've been using SBI calls to print.
    io::uart::use_as_console();

    // Setup and enable trap handler.
    irq::enable();

    // Handoff execution to the thread scheduler.
    task::executor::handoff(kmain, 0);
}

/// Main thread for the kernel.
extern "C" fn kmain(_: usize) -> isize {
    #[cfg(test)]
    crate::test_harness();

    // Enable external interrupts.
    irq::enable_external();

    sbi::reset::shutdown(sbi::reset::Reason::None);
}
