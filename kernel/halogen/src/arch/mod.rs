mod context;

pub use context::{Context, Privilege};

pub const TIMER_FREQ_HZ: usize = 10_000_000;


#[macro_export]
macro_rules! read_reg {
    ($reg:expr) => {
        {
            let reg: usize;
            unsafe { asm!(concat!("mv {}, ", stringify!($reg)), out(reg) reg); }
            reg
        }
    }
}

#[macro_export]
macro_rules! critical_section {
    { $($stmt:stmt)+ } => {
        #[allow(redundant_semicolon)]
        {
            let _cs = unsafe { riscv::interrupt::CriticalSection::new() };
            $($stmt)*
        }
    };
}
