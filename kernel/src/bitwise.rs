#[macro_export]
macro_rules! nth_bit {
    ($n:expr) => {
        (0b1 << $n)
    };
}

#[macro_export]
macro_rules! mask_lower {
    ($t: ty, $n:expr) => {
        ((0b1 as $t) << ($n + 1))
    };
}

#[macro_export]
macro_rules! mask_upper {
    ($t: ty, $n:expr) => {
        (((0b1 as $t) << ($n + 1)) ^ <$t>::MAX)
    };
}
