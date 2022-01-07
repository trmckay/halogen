pub const PTE_PER_PT: usize = 512;

#[macro_export]
macro_rules! sv39_enable {
    () => {{
        use riscv::register::satp;
        unsafe {
            satp::set(satp::Mode::Sv39, 0, 0);
        }
    }};
}

#[macro_export]
macro_rules! sv39_disable {
    () => {{
        use riscv::register::satp;
        unsafe {
            satp::set(satp::Mode::Bare, 0, 0);
        }
    }};
}

#[repr(C, packed)]
struct PageTable {
    entries: [u64; 512],
}
