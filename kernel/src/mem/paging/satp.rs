use core::{arch::asm, ptr::addr_of};

use super::PageTable;
use crate::mask_range;

/// Supported paging modes: bare and Sv39
pub enum Mode {
    Bare,
    Sv39,
}

/// Set the paging mode; make sure the PPN is set correctly
/// before calling
pub fn satp_set_mode(mode: Mode) {
    let mode_n = match mode {
        Mode::Bare => 0,
        Mode::Sv39 => 8,
    } << 60;
    let mut satp: usize;
    unsafe { asm!("csrr {}, satp", out(reg) satp) };
    satp = (satp & !mask_range!(usize, 63, 60)) | mode_n;
    unsafe { asm!("csrw satp, {}", in(reg) satp) };
}

/// Set the PPN field of the SATP register to point to the
/// given root page-table
pub fn satp_set_ppn(root: &PageTable) {
    let ppn = addr_of!(*root) as usize >> 12;
    let mut satp: usize;
    unsafe { asm!("csrr {}, satp", out(reg) satp) };
    satp = (satp & !mask_range!(usize, 63, 60)) | ppn;
    unsafe { asm!("csrw satp, {}", in(reg) satp) };
}
