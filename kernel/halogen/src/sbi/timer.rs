use super::{call::sbi_ecall, SbiError};

const TIMER_EXT_ID: usize = 0x54494D45;
const SET_TIMER_FUNC_ID: usize = 0;


pub fn set_timer(delay: isize) -> Result<usize, SbiError> {
    let time = match delay {
        -1 => usize::MAX,
        _ => riscv::register::time::read() + delay as usize,
    };

    let args = [time, 0, 0, 0, 0, 0];
    sbi_ecall(args, TIMER_EXT_ID, SET_TIMER_FUNC_ID)
}
