/// Write to a physical address
#[macro_export]
macro_rules! phys_write {
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

/// Read from a physical address
#[macro_export]
macro_rules! phys_read {
    ($a:expr) => {
        unsafe { ($a as *mut u8).read_volatile() }
    };
    ($a:expr, $o:expr) => {
        unsafe { ($a as *mut u8).add($o).read_volatile() }
    };
}
