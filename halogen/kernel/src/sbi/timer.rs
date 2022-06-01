use super::call::sbi_ecall;
use crate::{arch::TIMER_FREQ_HZ, log::*};

const TIMER_EXT_ID: usize = 0x54494D45;
const SET_TIMER_FUNC_ID: usize = 0;

fn us_to_cycles(us: usize) -> usize {
    us * TIMER_FREQ_HZ / 1_000_000
}

/// Set the timer such that it will trigger an interrupt in `delay_us`
/// microseconds. A delay of `usize::MAX` will disable the timer.
pub fn set(delay_us: usize) {
    let time = match delay_us {
        usize::MAX => usize::MAX,
        _ => {
            trace!("Set timer +{} us", delay_us);
            riscv::register::time::read().wrapping_add(us_to_cycles(delay_us))
        }
    };

    let args = [time, 0, 0, 0, 0, 0];
    sbi_ecall(TIMER_EXT_ID, SET_TIMER_FUNC_ID, args);
}
