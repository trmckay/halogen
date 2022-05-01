#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    exclusive_range_pattern,
    custom_test_frameworks,
    naked_functions,
    fn_align,
    asm_sym,
    asm_const,
    alloc_error_handler,
    stmt_expr_attributes,
    is_some_with
)]
#![allow(arithmetic_overflow)]
#![allow(dead_code)]
#![test_runner(crate::test::run_tests)]
#![reexport_test_harness_main = "test_harness"]

#[cfg(not(target_arch = "riscv64"))]
core::compile_error!("riscv64 is the only supported target_arch");

/// Architecture state and functionality
pub mod arch;
/// Entrypoint for OpenSBI
pub mod boot;
/// I/O devices
pub mod io;
/// Interrupt and trap handlers
pub mod irq;
/// Debug logging over UART
pub mod log;
/// Memory management
pub mod mem;
/// Panic language-feature
pub mod panic;
/// Crate-wide imports and definitions
pub mod prelude;
/// Interfacing with OpenSBI
pub mod sbi;
/// Scheduler API
pub mod sched;
/// System call definitions
pub mod syscall;
/// Unit and integration tests
pub mod test;
/// Kernel thread primitives
pub mod thread;
/// Bitwise manipulation utilities
pub mod util;

use prelude::*;

// TODO: The hart ID should be passed into/saved by `kmain()` and `HART_ID`
// should be hart-local and lock-free
pub static mut HART_ID: AtomicUsize = AtomicUsize::new(0);

#[macro_export]
macro_rules! hart_id {
    () => {
        #[allow(unused_unsafe)]
        unsafe {
            $crate::HART_ID.load(Ordering::Relaxed)
        }
    };
}

/// Entry-point for the kernel
///
/// # Safety
///
/// * `free_start` should point to a mapped, writeable, page-aligned, and free
///   address
/// * `free_size` should be page-aligned
#[allow(named_asm_labels)]
#[repr(align(4))]
pub unsafe extern "C" fn kinit(free_start: *mut u8, free_size: usize, page_offset: usize) -> ! {
    register::stvec::write(
        irq::trap::trap_shim as usize,
        register::stvec::TrapMode::Direct,
    );

    println_unsafe!("\n---\n");

    // Trace level logging on debug builds
    #[cfg(debug_assertions)]
    log::set_level(log::Level::Trace);

    trace!("Entered kinit, bootstrap complete");

    info!("Page offset: {:p}", page_offset as *const u8);
    info!("Free memory start: {:p}", free_start as *const u8);
    info!("Free memory size: {:.04}M", free_size as f32 / MIB as f32);

    // Initialize the physical frame allocator
    mem::frame_alloc_init(free_start as usize, free_size);

    // Save the page offset for later use
    *mem::paging::PAGE_OFFSET.lock() = page_offset;

    // Initialize the heap allocator
    let heap_addr = free_start.add(page_offset);
    mem::heap_alloc_init(heap_addr, mem::heap::HEAP_SIZE);

    // Initialize the stack allocator
    mem::stack_init(
        heap_addr.add(mem::heap::HEAP_SIZE),
        mem::stack::STACK_REGION_SIZE,
    );

    // Allocate trap-handler memory
    irq::setup();
    irq::enable();

    // Handoff execution to the thread scheduler
    thread::handoff(kmain, 0);
}

extern "C" fn kmain(_: usize) -> usize {
    trace!("Entered kmain");

    #[cfg(test)]
    crate::test_harness();

    // Enable external interrupts
    irq::enable_external();

    // Setup UART
    io::uart::enable_plic_int();
    io::uart::UART.lock().init();

    exit!(0);
}
