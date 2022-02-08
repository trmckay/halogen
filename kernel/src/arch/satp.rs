use core::{arch::asm, ptr::addr_of};

use crate::mem::PageTable;

/// Supported paging modes: bare and Sv39.
pub enum Mode {
    Bare,
    Sv39,
}

/// Set the paging mode. Make sure the PPN is set correctly
/// before calling.
pub fn bootstrap(
    mode: Mode,
    root: &PageTable,
    asid: usize,
    sp: usize,
    gp: usize,
    args: [usize; 8],
) {
    let mode_n = match mode {
        Mode::Bare => 0,
        Mode::Sv39 => 8,
    };
    let ppn = addr_of!(*root) as usize >> 12;
    let satp: usize = (mode_n << 60) | (asid << 44) | ppn;
    unsafe {
        asm!("
        mv sp, {}
        mv gp, {}
        csrw satp, {}
    ", in(reg) sp, in(reg) gp, in(reg) satp,
           inout("a0") args[0] => _,
           inout("a1") args[1] => _,
           inout("a2") args[2] => _,
           inout("a3") args[3] => _,
           inout("a4") args[4] => _,
           inout("a5") args[5] => _,
           inout("a6") args[6] => _,
           inout("a7") args[7] => _
        )
    };
}
