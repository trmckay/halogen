use halogen_common::mem::{Address, PhysicalAddress, KIB};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    critical_section,
    mem::{
        io::PLIC_BASE,
        paging::{map, Permissions},
    },
};


const INT_SOURCE_COUNT: usize = 1024;
const CONTEXT_COUNT: usize = 15872;

const ISR_COUNT: usize = 32;
const PRIORITY_MAX: u32 = 7;

static mut ISRS: [Option<InterruptRoutine>; ISR_COUNT] = [None; ISR_COUNT];

lazy_static! {
    static ref PLIC: Mutex<Plic> = unsafe { Mutex::new(Plic::new(PLIC_BASE)) };
}

pub fn register_isr(irq: usize, isr: InterruptRoutine) {
    unsafe {
        ISRS[irq] = Some(isr);
    }
}

pub fn set_enabled(irq: usize, enabled: bool) {
    critical_section! {
        PLIC.lock().set_enabled(0, irq, enabled);
    }
}

pub fn set_priority(irq: usize, priority: u32) {
    assert!(priority < PRIORITY_MAX);
    critical_section! {
        PLIC.lock().set_priority(irq, priority);
    }
}

pub fn set_threshold(threshold: u32) {
    critical_section! {
        PLIC.lock().set_threshold(0, threshold);
    }
}

/// Handle the next pending external interrupt mark it as complete. Returns
/// `None` if there are no interrupts pending.
pub fn handle_next() -> Option<usize> {
    None
}

/// One word per interrupt source
const PRIORITIES_OFFSET: isize = 0x0;
const PRIORITIES_LEN: usize = INT_SOURCE_COUNT * 4;

/// One bit per interrupt source
const PENDING_OFFSET: isize = 0x1000;
const PENDING_LEN: usize = INT_SOURCE_COUNT / 8;

/// One bit per interrupt source per hart context
const ENABLES_OFFSET: isize = 0x2000;
const ENABLES_LEN: usize = (INT_SOURCE_COUNT / 8) * CONTEXT_COUNT;

// 4K per context
const CONTEXT_OFFSET: isize = 0x200000;
const CONTEXT_LEN: usize = 4 * KIB * CONTEXT_COUNT;

pub type InterruptRoutine = fn() -> usize;

#[repr(C, packed)]
struct InterruptClaim {
    priority: u32,
    claim_complete: u32,
}

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
    unsafe fn new(base: PhysicalAddress) -> Plic {
        // For each section (priorites, enables, etc.) map the MMIO address and create a
        // slice
        let priorities = map(
            None,
            Some(base.add_offset(PRIORITIES_OFFSET)),
            PRIORITIES_LEN,
            Permissions::ReadWrite,
        )
        .expect("failed to map PLIC priorities")
        .as_mut_ptr();

        let pending = map(
            None,
            Some(base.add_offset(PENDING_OFFSET)),
            PENDING_LEN,
            Permissions::ReadOnly,
        )
        .expect("failed to map PLIC pending")
        .as_mut_ptr();

        let enables = map(
            None,
            Some(base.add_offset(ENABLES_OFFSET)),
            ENABLES_LEN,
            Permissions::ReadWrite,
        )
        .expect("failed to map PLIC enables")
        .as_mut_ptr();

        let contexts = map(
            None,
            Some(base.add_offset(CONTEXT_OFFSET)),
            CONTEXT_LEN,
            Permissions::ReadWrite,
        )
        .expect("failed to map PLIC enables")
        .as_mut_ptr();

        Plic {
            priorities,
            pending,
            enables,
            contexts,
        }
    }

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

    /// Set the priority for an interrupt
    fn set_priority(&self, irq: usize, priority: u32) {
        unsafe {
            (self.priorities as *mut u32)
                .add(irq)
                .write_volatile(priority);
        }
    }

    fn set_threshold(&self, hart: usize, threshold: u32) {
        unsafe {
            ((self.contexts as *mut [u8; 4 * KIB]).add(hart) as *mut u32).write_volatile(threshold);
        }
    }

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

    fn complete(&self, hart: usize, isr: u32) {
        unsafe {
            ((self.contexts as *mut [u8; 4 * KIB]).add(hart) as *mut u32)
                .add(1)
                .write_volatile(isr)
        }
    }
}
