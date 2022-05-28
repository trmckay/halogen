use super::call::sbi_ecall;
use crate::io::console::early_println;

const RESET_EXT_ID: usize = 0x53525354;
const RESET_FN_ID: usize = 0;

const SHUTDOWN: usize = 0;
const COLD_REBOOT: usize = 1;
const WARM_REBOOT: usize = 2;

pub enum Reason {
    None,
    Failure,
}

impl From<Reason> for usize {
    fn from(reason: Reason) -> usize {
        match reason {
            Reason::None => 0,
            Reason::Failure => 1,
        }
    }
}

/// Shutdown the platform
///
/// NOTE: qemu does not appear to propogate the exit code to the shell
///
/// # Safety
///
/// - Shutdown means shutdown
pub unsafe fn shutdown(reason: Reason) -> ! {
    match reason {
        Reason::None => early_println("\nShutdown"),
        Reason::Failure => early_println("\nShutdown due to error"),
    }
    sbi_ecall(
        RESET_EXT_ID,
        RESET_FN_ID,
        [SHUTDOWN, reason.into(), 0, 0, 0, 0],
    )
    .unwrap_unchecked();
    unreachable!();
}
