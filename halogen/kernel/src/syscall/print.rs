use core::{slice::from_raw_parts, str::from_utf8};

use crate::fwprint;

pub fn syscall_print(msg: *const u8, n: usize) -> isize {
    // Set SUM to read user memory.
    unsafe {
        riscv::register::sstatus::set_sum();
    }

    let msg = unsafe {
        match from_utf8(from_raw_parts(msg, n)) {
            Ok(s) => s,
            Err(_) => return 1,
        }
    };

    unsafe {
        riscv::register::sstatus::clear_sum();
    }

    fwprint!("{}", msg);

    0
}
