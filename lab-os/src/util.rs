#[macro_export]
macro_rules! bit {
    ($n:expr) => {
        (0b1 << $n)
    };
}
