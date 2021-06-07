#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
        use core::fmt::Write;

        #[cfg(machine="qemu")]
        {
            use crate::driver::{UartDriver, UartQemu};
            let mut uart = UartQemu::new();
            let _ = write!(uart, $($args)+);
        }
    });
}

#[macro_export]
macro_rules! println
{
    () => ({
        print!("\n")
    });
    ($fmt:expr) => ({
        print!(concat!($fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\n"), $($args)+)
    });
}
