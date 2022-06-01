use super::call::sbi_ecall;
use crate::fwprintln;

const HSM_EXT_ID: usize = 0x48534D;

const START_FN_ID: usize = 0;
const STOP_FN_ID: usize = 1;

/// Stop this hart and return control to the firmware.
///
/// # Safety
///
/// - If there is no hart to bring this back up, this effectively terminates the
///   kernel (`shutdown` would be a better choice).
pub unsafe fn hart_stop() -> ! {
    fwprintln!("Stopping hart");
    sbi_ecall(HSM_EXT_ID, STOP_FN_ID, [0; 6]);

    unreachable!()
}
