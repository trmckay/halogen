#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    exclusive_range_pattern,
    custom_test_frameworks,
    naked_functions,
    fn_align,
    asm_sym,
    alloc_error_handler
)]
#![allow(arithmetic_overflow)]
#![allow(dead_code)]
#![test_runner(crate::tests::run_tests)]
#![reexport_test_harness_main = "test_harness"]

#[cfg(not(target_arch = "riscv64"))]
core::compile_error!("Halogen only supports riscv64");

/// Architecture specifics
pub mod arch;
/// Entrypoint for OpenSBI
mod boot;
/// Logging utilities
pub mod log;
/// Memory management
pub mod mem;
/// Panic language-feature
mod panic;
/// Interfacing with OpenSBI
pub mod sbi;
/// Unit and integration tests
#[cfg(test)]
mod tests;
/// Trap handlers
mod trap;
/// Bitwise manipulation utilities
pub mod util;

const MOTD: &str = r"
 _   _       _
| | | | __ _| | ___   __ _  ___ _ __
| |_| |/ _` | |/ _ \ / _` |/ _ | '_ \
|  _  | (_| | | (_) | (_| |  __| | | |
|_| |_|\__,_|_|\___/ \__, |\___|_| |_|
                     |___/
";

use core::arch::asm;

/// Entry-point for the kernel. The arguments are the beginning virtual
/// address and size of the memory available to the kernel.
///
/// # Safety
///
/// `free_start` should point to a mapped, writeable, page-aligned, and free
/// address. `free_size` should be page-aligned.
#[allow(named_asm_labels)]
#[repr(align(4))]
pub unsafe extern "C" fn kmain(free_start: usize, free_size: usize, page_offset: usize) -> ! {
    asm!("csrw stvec, {}", in(reg) trap::trap_handler);

    log::LOGGER.register();

    println!("{}", MOTD);

    mem::frame_alloc_init(free_start, free_size);
    log::info!(
        "Available memory: {}M @ {:p}",
        free_size / (1024 * 1024),
        free_start as *const u8
    );

    *mem::paging::PAGE_OFFSET.lock() = page_offset;
    log::info!(
        "Page offset: {:p}",
        *mem::paging::PAGE_OFFSET.lock() as *const u8
    );

    let heap_addr = free_start + page_offset;
    let stack_addr = heap_addr + mem::heap::HEAP_SIZE;

    mem::kstack_init(stack_addr);

    mem::heap_alloc_init(heap_addr);

    #[cfg(test)]
    crate::test_harness();

    exit!(0);
}
