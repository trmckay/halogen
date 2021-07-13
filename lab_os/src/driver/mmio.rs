/// Write to an MMIO address.
///
/// Examples:
///
/// ```
/// // Write 17 to address 0x1000.
/// mmio_wr!(0x1000, 17)
///
/// // Write 17 to address 0x1004.
/// mmio_wr!(0x1000, 0x4, 17)
/// ```
#[macro_export]
macro_rules! mmio_wr {
    ($a:expr, $d:expr) => {
        unsafe {
            ($a as *mut u8).write_volatile($d);
        }
    };
    ($a:expr, $o:expr, $d:expr) => {
        unsafe {
            ($a as *mut u8).add($o).write_volatile($d);
        }
    };
}

/// Read from an MMIO address.
///
/// Examples:
///
/// ```
/// // Read from address 0x1000.
/// mmio_wr!(0x1000, 17)
///
/// // Read from address 0x1004.
/// mmio_wr!(0x1000, 0x4, 17)
/// ```
#[macro_export]
macro_rules! mmio_rd {
    ($a:expr) => {
        unsafe { ($a as *mut u8).read_volatile() }
    };
    ($a:expr, $o:expr) => {
        unsafe { ($a as *mut u8).add($o).read_volatile() }
    };
}
