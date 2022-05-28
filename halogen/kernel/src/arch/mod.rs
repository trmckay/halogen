mod context;
use core::sync::atomic::AtomicUsize;

pub use context::{Context, Privilege};

pub const TIMER_FREQ_HZ: usize = 10_000_000;

// TODO: The hart ID should be passed into/saved by `kmain()` and `HART_ID`
// should be hart-local and lock-free to read.
pub static mut HART_ID: AtomicUsize = AtomicUsize::new(0);

/// Maps register numbers to names.
pub const REGISTER_NAMES: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0/fp", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
    "t5", "t6",
];

/// Get the hart ID of the caller.
#[macro_export]
macro_rules! hart_id {
    () => {
        #[allow(unused_unsafe)]
        unsafe {
            $crate::arch::HART_ID.load(Ordering::Relaxed)
        }
    };
}

/// Read a general purpose register.
#[macro_export]
macro_rules! read_reg {
    ($reg:expr) => {
        {
            let reg: usize;
            #[allow(unused_unsafe)]
            unsafe {
                core::arch::asm!(concat!("mv {}, ", stringify!($reg)), out(reg) reg);
            }
            reg
        }
    }
}

/// Read a control register with `csrr`.
#[macro_export]
macro_rules! read_csr {
    ($reg:expr) => {
        {
            let reg: usize;
            #[allow(unused_unsafe)]
            unsafe {
                core::arch::asm!(concat!("csrr {}, ", stringify!($reg)), out(reg) reg);
            }
            reg
        }
    }
}

/// Disable interrupts for a block statement.
#[macro_export]
macro_rules! critical_section {
    { $($stmt:stmt)+ } => {
        #[allow(redundant_semicolons)]
        {
            #[allow(unused_unsafe)]
            let _cs = unsafe { riscv::interrupt::CriticalSection::new() };
            $($stmt)*
        }
    };
}
