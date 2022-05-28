use super::{call::sbi_ecall, error::SbiError};
use crate::log::*;

const TIMER_EXT_ID: usize = 0x54494D45;
const SET_TIMER_FUNC_ID: usize = 0;


pub fn set(delay: isize) -> Result<usize, SbiError> {
    let time = match delay {
        -1 => usize::MAX,
        _ => riscv::register::time::read() + delay as usize,
    };

    trace!("Set timer +{}", delay);

    let args = [time, 0, 0, 0, 0, 0];
    sbi_ecall(TIMER_EXT_ID, SET_TIMER_FUNC_ID, args)
}
