/// Macro to generate a bitmask for just the nth bit
///
/// ```
/// assert_eq!(0b100, crate::nth_bit!(2));
/// ```
#[macro_export]
macro_rules! nth_bit {
    ($n:expr) => {
        (0b1 << $n)
    };
}

/// Create a bitmask for the `$msb` to `$lsb` bits
#[macro_export]
macro_rules! mask_range {
    ($t:ty, $msb:expr, $lsb:expr) => {
        (((0b1 as $t) << ($msb - $lsb + 1)) - 1) << $lsb
    };
}
