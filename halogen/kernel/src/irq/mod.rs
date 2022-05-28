use crate::log::*;

pub mod plic;

/// Enable external interrupts
pub fn enable_external() {
    info!("Enable external interrupts");
    plic::set_threshold(0);
    unsafe {
        riscv::register::sie::set_sext();
    }
}

/// Enable timer interrupts
#[inline]
pub fn enable_timer() {
    unsafe {
        riscv::register::sie::set_stimer();
    }
}

/// Disable all interrupts
#[inline]
pub fn disable() {
    unsafe {
        riscv::register::sstatus::clear_sie();
    }
}

/// Disable all interrupts
#[inline]
pub fn enable() {
    unsafe {
        riscv::register::sstatus::set_sie();
    }
}
