//! Physical addresses as sizes for mapping devices
//!
//! Sizes are in units of pages

use halogen_common::mem::PhysicalAddress;

pub const UART_BASE: PhysicalAddress = PhysicalAddress(0x1000_0000);
pub const UART_SIZE: usize = 1;

pub const PLIC_BASE: PhysicalAddress = PhysicalAddress(0x0C00_0000);
