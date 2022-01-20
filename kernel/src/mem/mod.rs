mod kmalloc;
pub mod paging;

pub use kmalloc::kmalloc;
pub use paging::*;

extern "C" {
    pub static KERNEL_END: usize;
}

#[macro_export]
macro_rules! size_of {
    ($t:tt) => {
        (core::mem::size_of::<$t>())
    };
}
