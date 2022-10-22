/// Aligns an address to a boundary (power of 2)
///
/// # Example
///
/// ```
/// assert_eq!(64, halogen_lib::align_up!(63, 64_usize));
/// assert_eq!(0, halogen_lib::align_up!(0, 64_usize));
/// assert_eq!(8192, halogen_lib::align_up!(8192, 4096_usize));
/// ```
#[macro_export]
macro_rules! align_up {
    ($n:expr, $d:expr) => {{
        assert!($d.is_power_of_two());
        ($n + ($d - 1)) & !($d - 1)
    }};
}

/// Create a bitmask for the `$msb` to `$lsb` bits.
#[macro_export]
macro_rules! mask {
    ($lsb:expr, $msb:expr) => {
        ((0b1 << ($msb - $lsb + 1)) - 1) << $lsb
    };
}

/// Align an address down to a boundary.
///
/// # Example
///
/// ```
/// assert_eq!(64, halogen_lib::align_down!(69, 64_usize));
/// assert_eq!(0, halogen_lib::align_down!(12, 64_usize));
/// assert_eq!(8192, halogen_lib::align_down!(8192, 4096_usize));
/// ```
#[macro_export]
macro_rules! align_down {
    ($n:expr, $d:expr) => {
        $n - ($n % $d)
    };
}

/// Evaluate to true if an address is aligned to a boundary.
#[macro_export]
#[allow(clippy::nonminimal_bool)]
macro_rules! is_aligned {
    ($addr:expr, $d:expr) => {
        0 == $addr % $d
    };
}

/// Base 2 logarithm.
#[macro_export]
macro_rules! log2_ceil {
    ($n:expr) => {{
        let mut b = 0;
        let mut n = $n >> 1;

        while (n > 0) {
            n >>= 1;
            b += 1;
        }

        b
    }};
}

/// Base 2 logarithm.
#[macro_export]
macro_rules! div_ceil {
    ($a:expr, $b:expr) => {
        ($a + $b - 1) / $b
    };
}

/// Clamp an expression between a min and max.
#[macro_export]
macro_rules! clamp {
    ($n:expr, $max:expr) => {
        if $n > $max {
            $max
        } else {
            $n
        }
    };
    ($n:expr, $min:expr, $max:expr) => {
        if $n > $max {
            $max
        } else if $n < $min {
            $min
        } else {
            $n
        }
    };
}
