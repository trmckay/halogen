use super::*;

const TIMER_EXT_ID: usize = 0x54494D45;
const SET_TIMER_FUNC_ID: usize = 0;

fn read_time() -> usize {
    let time;
    unsafe {
        asm!("csrr {}, time", out(reg) time);
    }
    time
}

pub fn set_timer(delay: isize) -> Result<usize, SbiError> {
    let time = match delay {
        -1 => usize::MAX,
        _ => read_time() + delay as usize,
    };

    let args = [time, 0, 0, 0, 0, 0];
    sbi_ecall(args, TIMER_EXT_ID, SET_TIMER_FUNC_ID)
}
