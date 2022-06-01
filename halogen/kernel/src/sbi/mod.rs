/// Make a call to firmware with the SBI ABI.
pub(self) mod call;

/// Firmware console implementation.
pub mod console;
/// Hart-state management.
pub mod hsm;
/// Platform shutdown/reset.
pub mod reset;
/// Set the supervisor timer interrupt.
pub mod timer;
