/// Make a call to the supporting environment, an M-mode firmware that
/// implements the SBI spec.
pub fn sbi_ecall(ext: usize, func: usize, args: [usize; 6]) -> usize {
    let ret_code: isize;
    let val: usize;

    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") args[0],
            in("a1") args[1],
            inout("a2") args[2] => _,
            inout("a3") args[3] => _,
            inout("a4") args[4] => _,
            inout("a5") args[5] => _,
            inout("a6") func => _,
            inout("a7") ext => _,
            lateout("a0") ret_code,
            lateout("a1") val,
        );
    }

    assert!(ret_code >= 0);
    val
}
