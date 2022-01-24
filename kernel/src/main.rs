#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    exclusive_range_pattern,
    custom_test_frameworks,
    naked_functions,
    fn_align,
    asm_sym
)]
#![allow(arithmetic_overflow)]
#![allow(dead_code)]
#![test_runner(crate::kmain_test::run_tests)]
#![reexport_test_harness_main = "test_harness"]

#[cfg(not(target_arch = "riscv64"))]
core::compile_error!("Halogen only supports riscv64");

#[cfg(not(test))]
pub use crate::kmain::kmain;
#[cfg(test)]
pub use crate::kmain_test::kmain;

/// Entrypoint for OpenSBI
mod boot;
/// Memory management
mod mem;
/// Panic language-feature
mod panic;
/// Interfacing with OpenSBI
mod sbi;
/// Bitwise manipulation utilities
mod util;

mod arch;

const MOTD: &str = r"
 _   _       _
| | | | __ _| | ___   __ _  ___ _ __
| |_| |/ _` | |/ _ \ / _` |/ _ | '_ \
|  _  | (_| | | (_) | (_| |  __| | | |
|_| |_|\__,_|_|\___/ \__, |\___|_| |_|
                     |___/
";

#[cfg(not(test))]
mod kmain {
    use super::*;

    /// Entry-point for the kernel. After the assembly-based set-up
    /// is complete, the system will jump here
    #[no_mangle]
    #[allow(named_asm_labels)]
    // stvec must be 4-byte aligned
    #[repr(align(4))]
    pub extern "C" fn kmain() -> ! {
        println!("{}", MOTD);
        unimplemented!();
    }
}

#[cfg(test)]
mod kmain_test {
    use super::*;

    #[no_mangle]
    #[allow(named_asm_labels)]
    #[repr(align(4))]
    pub extern "C" fn kmain() -> ! {
        crate::test_harness();
        exit!(0);
    }

    pub fn run_tests(tests: &[&dyn Fn()]) {
        println!("{}", MOTD);
        println!("\nRunning {} tests", tests.len());
        for test in tests {
            test();
        }
    }
}
