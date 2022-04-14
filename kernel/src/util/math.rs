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

/// Aligns an address to a boundary
#[macro_export]
macro_rules! align {
    ($n:expr, $d:expr) => {
        ($n + ($d - 1)) & !($d - 1)
    };
}

/// Evaluates to true if an address is aligned to a boundary
#[macro_export]
#[allow(clippy::nonminimal_bool)]
macro_rules! is_aligned {
    ($addr:expr, $d:expr) => {{
        (0 == $addr % $d)
    }};
}

/// Base 2 logarithm
#[macro_export]
macro_rules! log2 {
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

/// Clamp an expression between a min and max
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
