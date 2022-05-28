use crate::log::*;

/// Interface with a PLIC device that implements the RISC-V PLIC spec.
pub mod plic;

/// Enable supervisor external interrupts.
pub fn enable_external() {
    info!("Enable external interrupts");
    plic::set_threshold(0);
    unsafe {
        riscv::register::sie::set_sext();
    }
}

/// Enable timer interrupts.
#[inline]
pub fn enable_timer() {
    unsafe {
        riscv::register::sie::set_stimer();
    }
}

/// Disable all interrupts.
#[inline]
pub fn disable() {
    unsafe {
        riscv::register::sstatus::clear_sie();
    }
}

/// Enable all interrupts.
#[inline]
pub fn enable() {
    unsafe {
        riscv::register::sstatus::set_sie();
    }
}
