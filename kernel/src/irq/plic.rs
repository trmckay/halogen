use core::sync::atomic::AtomicUsize;

use crate::{mem, prelude::*};

const PLIC_PRIORITY_OFFSET: usize = 0x0;
const PLIC_PENDING_OFFSET: usize = 0x1000;
const PLIC_ENABLE_OFFSET: usize = 0x2000;
const PLIC_MACHINE_OFFSET: usize = 0x0;
const PLIC_CTX_STEP: usize = 0x80;

const PLIC_CTX_THRESHOLD_OFFSET: usize = 0x0;
const PLIC_CTX_CLAIM_OFFSET: usize = 0x4;
const PLIC_CTX_MACHINE_OFFSET: usize = 0x0;
const PLIC_CTX_SUPERVISOR_OFFSET: usize = 0x1000;
pub const PLIC_CTX_HART_STEP: usize = 0x2000;

const ISR_COUNT: usize = 32;
const PRIORITY_MAX: u32 = 7;

#[allow(clippy::declare_interior_mutable_const)]
const ATOMIC_ZERO: AtomicUsize = AtomicUsize::new(0);

static ISR_TABLE: [AtomicUsize; ISR_COUNT] = [ATOMIC_ZERO; ISR_COUNT];

/// Pointer to the enable bits in the PLIC for the specified hart ID
macro_rules! enables {
    ($hid:expr) => {
        // Machine and supervisor contexts for each hart; offset 2 contexts per hart,
        // then one more for supervisor
        (mem::DEV_PLIC + mem::MMIO_OFFSET + PLIC_ENABLE_OFFSET + (PLIC_CTX_STEP * (2 * $hid + 1)))
            as *mut u32
    };
}

/// Pointer to the priorities bits in the PLIC for the specified hart ID
macro_rules! priorities {
    ($irq:expr) => {
        ((mem::DEV_PLIC + mem::MMIO_OFFSET + PLIC_PRIORITY_OFFSET + (4 * $irq)) as *mut u32)
    };
}

/// Pointer to the claims bits in the PLIC for the specified hart ID
macro_rules! claims {
    ($hid:expr) => {
        (mem::DEV_PLIC_CONTEXT
            + mem::MMIO_OFFSET
            + PLIC_CTX_CLAIM_OFFSET
            + PLIC_CTX_SUPERVISOR_OFFSET
            + (PLIC_CTX_HART_STEP * $hid)) as *mut u32
    };
}

/// Pointer to the thresholds bits in the PLIC for the specified hart ID
macro_rules! thresholds {
    ($hid:expr) => {
        (mem::DEV_PLIC_CONTEXT
            + mem::MMIO_OFFSET
            + PLIC_CTX_THRESHOLD_OFFSET
            + PLIC_CTX_SUPERVISOR_OFFSET
            + (PLIC_CTX_HART_STEP * $hid)) as *mut u32
    };
}

/// Enable an interrupt on this hart
pub fn enable(irq: u32) {
    let irq = clamp!(irq as usize, 1, ISR_COUNT - 1);
    let enables = enables!(hart_id!());
    unsafe { enables.write_volatile(enables.read_volatile() | (0b1 << irq)) };
}

/// Disable an interrupt on this hart
pub fn disable(irq: u32) {
    let irq = clamp!(irq as usize, 1, ISR_COUNT - 1);
    let enables = enables!(hart_id!());
    unsafe { enables.write_volatile(enables.read_volatile() & !(0b1 << irq)) };
}

/// Get the status of an IRQ; `true` is enabled
pub fn status(irq: u32) -> bool {
    let irq = clamp!(irq as usize, 1, ISR_COUNT - 1);
    let enables = enables!(hart_id!());
    unsafe { enables.read_volatile() & (0b1 << irq) != 0 }
}

/// Set the global priority for an interrupt
pub fn set_priority(irq: u32, priority: u32) {
    unsafe { priorities!(irq as usize).write_volatile(clamp!(priority, PRIORITY_MAX)) };
}

/// Set the interrupt threshold for this hart
pub fn set_threshold(thresh: u32) {
    unsafe { thresholds!(hart_id!()).write_volatile(clamp!(thresh, PRIORITY_MAX)) };
}

/// Register a function pointer as the handler for a interrupt request
pub fn register_isr(irq: u32, isr: fn() -> ()) {
    let ent = &ISR_TABLE[irq as usize];
    ent.store(isr as usize, core::sync::atomic::Ordering::Relaxed);
}

/// Complete an IRQ
pub fn complete(irq: u32) {
    unsafe { claims!(hart_id!()).write_volatile(irq) }
}

/// Get the next pending interrupt
pub fn pending() -> Option<u32> {
    match unsafe { claims!(hart_id!()).read_volatile() } {
        0 => None,
        id => Some(id),
    }
}

/// Get the hander function for an IRQ
pub fn handler(irq: u32) -> Option<fn() -> ()> {
    let ent = &ISR_TABLE[irq as usize];
    match ent.load(core::sync::atomic::Ordering::Relaxed) {
        0 => None,
        fn_ptr => Some(unsafe { transmute(fn_ptr) }),
    }
}

pub fn handle_pending() {
    if let Some(irq) = pending() {
        match handler(irq) {
            Some(handler) => {
                handler();
            }
            None => panic!("unregistered IRQ"),
        }
        complete(irq);
    }
}
