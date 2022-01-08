/// Count of page-table entries per page-table
pub const PTE_PER_PT: usize = 512;

/// Enable paging in Sv39 mode
#[macro_export]
macro_rules! sv39_enable {
    () => {{
        use riscv::register::satp;
        unsafe {
            satp::set(satp::Mode::Sv39, 0, 0);
        }
    }};
}

/// Set the paging mode to bare
#[macro_export]
macro_rules! sv39_disable {
    () => {{
        use riscv::register::satp;
        unsafe {
            satp::set(satp::Mode::Bare, 0, 0);
        }
    }};
}

/// Packed structure representing a single page-table
#[repr(C, packed)]
struct PageTable {
    entries: [u64; 512],
}
