/// Macros for accessing MMIO.
mod mmio;

// Platform-specific definitions should be collected in a
// submodule of `driver`, i.e. `driver::virt`.

/// Definitions for the QEMU virt machine.
mod virt;
pub use virt::DEV_UART;

/// Driver for UART. Assumes MMIO interface for reading
/// and writing.
mod uart;
pub use uart::UartDriver;
