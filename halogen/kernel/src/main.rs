#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    exclusive_range_pattern,
    const_maybe_uninit_zeroed,
    custom_test_frameworks,
    naked_functions,
    fn_align,
    asm_const,
    alloc_error_handler,
    stmt_expr_attributes,
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

mod boot;
mod panic;

pub mod arch;
pub mod error;
pub mod io;
pub mod irq;
pub mod log;
pub mod mem;
pub mod sbi;
pub mod syscall;
pub mod task;
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

// Main thread for the kernel.
extern "C" fn kmain(_: usize) -> isize {
    #[cfg(test)]
    crate::test_harness();

    // Enable external interrupts.
    irq::enable_external();

    sbi::reset::shutdown(sbi::reset::Reason::None);
}
