//! Physical addresses of memory-mapped I/O.

use halogen_lib::mem::PhysicalAddress;

/// Base address of the UART device.
pub const UART_BASE: PhysicalAddress = PhysicalAddress(0x1000_0000);
/// Size in pages of the UART memory map.
pub const UART_SIZE: usize = 1;

/// Base PLIC device.
pub const PLIC_BASE: PhysicalAddress = PhysicalAddress(0x0C00_0000);
