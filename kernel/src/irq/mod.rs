pub mod plic;
pub mod trap;

use crate::{mem::Stack, prelude::*};

/// Allocate memory for the trap handler
pub fn setup() {
    // Allocate a stack for the interrupt vector
    let trap_stack = Stack::new().expect("irq: could not allocate stack for trap handler");
    register::sscratch::write(trap_stack.top() as usize);

    // Don't run the destructor
    forget(trap_stack);
}

/// Enable external interrupts
pub fn enable_external() {
    plic::set_threshold(0);
    unsafe {
        register::sie::set_sext();
    }
}

/// Enable timer interrupts
#[inline(always)]
pub fn enable_timer() {
    unsafe {
        register::sie::set_stimer();
    }
}

/// Disable all interrupts
#[inline(always)]
pub fn disable() {
    unsafe {
        register::sstatus::clear_sie();
    }
}

/// Disable all interrupts
#[inline(always)]
pub fn enable() {
    unsafe {
        register::sstatus::set_sie();
    }
}
