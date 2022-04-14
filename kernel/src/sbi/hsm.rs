use super::*;

const HSM_EXT_ID: usize = 0x48534D;
const HSM_FUNC_STOP: usize = 1;

pub unsafe fn stop() -> ! {
    sbi_ecall([0; 6], HSM_FUNC_STOP, HSM_EXT_ID).expect("could not stop hart");
    unreachable!("hart did not stop");
}
