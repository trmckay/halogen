/// A macro to generate a bitmask for just the nth bit
///
/// ```
/// use crate::nth_bit;
///
/// assert_eq!(0b100, nth_bit!(2));
/// ```
#[macro_export]
macro_rules! nth_bit {
    ($n:expr) => {
        (0b1 << $n)
    };
}

/// A macro to generate a bitmask of the lower n bits of the specified type
///
/// ```
/// use crate::mask_lower;
///
/// assert_eq!(0b00111 as u64, mask_lower!(u64, 3));
/// ```
#[macro_export]
macro_rules! mask_lower {
    ($t: ty, $n:expr) => {
        ((0b1 as $t) << ($n + 1))
    };
}

/// A macro to generate a bitmask of the upper n bits
///
/// ```
/// use crate::mask_upper;
///
/// assert_eq!(0xF0, mask_upper!(u8, 4));
/// ```
#[macro_export]
macro_rules! mask_upper {
    ($t: ty, $n:expr) => {
        (((0b1 as $t) << ($n + 1)) ^ <$t>::MAX)
    };
}
