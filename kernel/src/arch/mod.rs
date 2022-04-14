mod context;

pub use context::Context;

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
