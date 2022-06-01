//! This module provides an interface to the platform level interrupt controller
//! (PLIC). This PLIC can be used to poll for and handle external interrupts.
//!
//! When there is an external interrupt, the hart will receive a supervisor
//! external interrupt. It should then check the PLIC to get the highest
//! priority IRQ. To handle the request, the hart will place a claim
//! on that IRQ, handle the request, mark it as completed, and finally return
//! from the trap handler.

use halogen_common::mem::{Address, PhysicalAddress, VirtualAddress, KIB};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    critical_section,
    mem::{
        io::PLIC_BASE,
        paging::{map, Permissions, Privilege, Scope},
    },
};


const INT_SOURCE_COUNT: usize = 1024;
const CONTEXT_COUNT: usize = 15872;

const ISR_COUNT: usize = 32;
const PRIORITY_MAX: u32 = 7;

static mut ISRS: [Option<InterruptRoutine>; ISR_COUNT] = [None; ISR_COUNT];

lazy_static! {
    static ref PLIC: Mutex<Plic> = unsafe { Mutex::new(Plic::from_phys(PLIC_BASE)) };
}

/// Register a function as the interrupt service routine for a given interrupt
/// source. This will be called when the IRQ is returned as pending.
pub fn register_isr(irq: usize, isr: InterruptRoutine) {
    unsafe {
        ISRS[irq] = Some(isr);
    }
}

/// Enable an interrupt source on the calling hart's context.
pub fn set_enabled(irq: usize, enabled: bool) {
    critical_section!({ PLIC.lock().set_enabled(0, irq, enabled) });
}

/// Set the priority of an interrupt source on the calling hart's context.
pub fn set_priority(irq: usize, priority: u32) {
    assert!(priority < PRIORITY_MAX);
    critical_section!({ PLIC.lock().set_priority(irq, priority) });
}

/// Set interrupt priority threshold for the calling hart's context.
pub fn set_threshold(threshold: u32) {
    critical_section!({ PLIC.lock().set_threshold(0, threshold) });
}

/// Handle the next pending external interrupt mark it as complete. Returns
/// `None` if there are no interrupts pending.
pub fn handle_next() -> Option<usize> {
    None
}

// One word per interrupt source.
const PRIORITIES_OFFSET: isize = 0x0;
const PRIORITIES_LEN: usize = INT_SOURCE_COUNT * 4;

// One bit per interrupt source.
const PENDING_OFFSET: isize = 0x1000;
const PENDING_LEN: usize = INT_SOURCE_COUNT / 8;

// One bit per interrupt source per hart context.
const ENABLES_OFFSET: isize = 0x2000;
const ENABLES_LEN: usize = (INT_SOURCE_COUNT / 8) * CONTEXT_COUNT;

// 4K per context.
const CONTEXT_OFFSET: isize = 0x200000;
const CONTEXT_LEN: usize = 4 * KIB * CONTEXT_COUNT;

/// A function that can be called as an ISR. Accepts no arguments and returns 0
/// on success.
pub type InterruptRoutine = fn() -> usize;

/// Platform-level interrupt controller per the RISC-V PLIC spec. The fields are
/// contiguous in physical memory, but this does not necessarily hold after the
/// struct is created and the fields are mapped.
struct Plic {
    priorities: *mut u8,
    pending: *mut u8,
    enables: *mut u8,
    contexts: *mut u8,
}

unsafe impl Sync for Plic {}
unsafe impl Send for Plic {}

impl Plic {
    /// Map the PLIC at the base physical address.
    unsafe fn from_phys(base: PhysicalAddress) -> Plic {
        // For each section (priorites, enables, etc.) map the MMIO address and create a
        // slice
        let priorities = map(
            None,
            Some(base.add_offset(PRIORITIES_OFFSET)),
            PRIORITIES_LEN,
            Permissions::ReadWrite,
            Scope::Global,
            Privilege::Kernel,
        )
        .expect("failed to map PLIC priorities");

        let pending = map(
            None,
            Some(base.add_offset(PENDING_OFFSET)),
            PENDING_LEN,
            Permissions::ReadOnly,
            Scope::Global,
            Privilege::Kernel,
        )
        .expect("failed to map PLIC pending");

        let enables = map(
            None,
            Some(base.add_offset(ENABLES_OFFSET)),
            ENABLES_LEN,
            Permissions::ReadWrite,
            Scope::Global,
            Privilege::Kernel,
        )
        .expect("failed to map PLIC enables");

        let contexts = map(
            None,
            Some(base.add_offset(CONTEXT_OFFSET)),
            CONTEXT_LEN,
            Permissions::ReadWrite,
            Scope::Global,
            Privilege::Kernel,
        )
        .expect("failed to map PLIC enables");

        Plic::from_virt(priorities, pending, enables, contexts)
    }

    /// Construct a PLIC from mapped regions.
    unsafe fn from_virt(
        priorities: VirtualAddress,
        pending: VirtualAddress,
        enables: VirtualAddress,
        contexts: VirtualAddress,
    ) -> Plic {
        Plic {
            priorities: priorities.as_mut_ptr(),
            pending: pending.as_mut_ptr(),
            enables: enables.as_mut_ptr(),
            contexts: contexts.as_mut_ptr(),
        }
    }

    /// Enable an interrupt source on hart.
    fn set_enabled(&self, hart: usize, irq: usize, enabled: bool) {
        // Two
        let index = (irq & 0xFFFF_FF00) >> 8;
        let offset = irq & 0xFF;

        unsafe {
            let entry = ((self.enables as *mut [u32; INT_SOURCE_COUNT / 8]).add(hart) as *mut u8)
                .add(index);

            if enabled {
                entry.write_volatile(entry.read_volatile() | (1 << offset));
            } else {
                entry.write_volatile(entry.read_volatile() & !(1 << offset));
            }
        }
    }

    /// Set the priority for an interrupt source.
    fn set_priority(&self, irq: usize, priority: u32) {
        unsafe {
            (self.priorities as *mut u32)
                .add(irq)
                .write_volatile(priority);
        }
    }

    /// Set the priority threshold for a hart.
    fn set_threshold(&self, hart: usize, threshold: u32) {
        unsafe {
            ((self.contexts as *mut [u8; 4 * KIB]).add(hart) as *mut u32).write_volatile(threshold);
        }
    }

    /// Claim an interrupt for a hart.
    fn claim(&self, hart: usize) -> Option<u32> {
        unsafe {
            match ((self.contexts as *mut [u8; 4 * KIB]).add(hart) as *mut u32)
                .add(1)
                .read_volatile()
            {
                0 => None,
                id => Some(id),
            }
        }
    }

    /// Complete an interrupt on a hart.
    fn complete(&self, hart: usize, isr: u32) {
        unsafe {
            ((self.contexts as *mut [u8; 4 * KIB]).add(hart) as *mut u32)
                .add(1)
                .write_volatile(isr)
        }
    }
}
