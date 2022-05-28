//! This module describes the kernel address-space layout. They are calculated
//! based on the constants and static variables (set during boot).

use halogen_common::mem::{Address, PhysicalAddress, Segment, VirtualAddress, GIB};

pub use super::addr_space::AddressSpace;
use super::paging::PAGE_SIZE;

/// Upper region of a 39-bit address-space. This is calculated as:
///
/// - `2^38` = `0x40_0000_0000` is the halfway point of the address-space.
/// - Bits 63-39 must match bit 38, so set these bits to 1.
/// - `0x40_0000_0000 + 0xFFFF_FF80_0000_0000`.
pub const KERNEL_SPACE_START: VirtualAddress = VirtualAddress(0xFFFF_FFC0_0000_0000);

/// The size of the address-space available to the kernel (upper-half of
/// 39-bits).
pub const KERNEL_SPACE_SIZE: usize = 1 << 38;

/// Each thread (kernel or user) has a kernel stack with at most 8 MiB to map.
pub const STACK_SIZE: usize = 8 * GIB;

/// Size of the shared kernel heap.
pub const HEAP_SIZE: usize = 8 * GIB;

/// The base address of physical memory.
pub static mut PHYSICAL_BASE: PhysicalAddress = PhysicalAddress(0);

/// Size of the physical memory
pub static mut PHYSICAL_SIZE: usize = 0;

/// Size of the physical memory
pub static mut FREE_SIZE: usize = 0;

/// Size of the read-execute section of the kernel image (set during bootstrap)
pub static mut TEXT_SIZE: usize = 0;
/// Size of the read-only section of the kernel image (set during bootstrap)
pub static mut RODATA_SIZE: usize = 0;
/// Size of the read-write section of the kernel image (set during bootstrap)
pub static mut RWDATA_SIZE: usize = 0;

/// Get the virtual offset from physical memory. If the virtual base is greater
/// than the physical base (almost always the case), this is positive.
#[inline]
pub fn virtual_offset() -> isize {
    KERNEL_SPACE_START.offset(unsafe { PHYSICAL_BASE })
}

/// Provides an easier way to declare region constants.
macro_rules! region {
    ($name:ident, $start:expr, $size:expr) => {
        lazy_static::lazy_static! {
            pub static ref $name: Segment<VirtualAddress> = Segment::from_size($start, $size);
        }
    };
}

// The bounds of these regions are calculated on first access (`lazy_static`)
// based on the constants above set during bootstrap.
//
// TODO: This could be done nicely with a TT-muncher macro.

region!(IMAGE_TEXT, KERNEL_SPACE_START, unsafe { TEXT_SIZE });
region!(IMAGE_RO, IMAGE_TEXT.end, unsafe { RODATA_SIZE });
region!(IMAGE_RW, IMAGE_RO.end, unsafe { RWDATA_SIZE });
region!(FRAME_ALLOC, IMAGE_RW.end, unsafe { FREE_SIZE });
region!(STACK, IMAGE_RW.end, STACK_SIZE);
region!(HEAP, STACK.end, HEAP_SIZE);
region!(VIRT_SPACE, HEAP.end, VirtualAddress(usize::MAX) - HEAP.end);

/// Regions of the kernel address-space.
#[derive(Debug, Copy, Clone)]
pub enum Region {
    Zero,
    User,
    ImageText,
    ImageRo,
    ImageRw,
    Heap,
    Stack,
    Physical,
    Dynamic,
}

impl From<VirtualAddress> for Region {
    fn from(virt_addr: VirtualAddress) -> Region {
        let virt_addr_i: usize = virt_addr.into();
        if virt_addr_i <= PAGE_SIZE {
            Region::Zero
        } else if virt_addr_i < KERNEL_SPACE_START.into() {
            Region::User
        } else if IMAGE_TEXT.contains(virt_addr) {
            Region::ImageText
        } else if IMAGE_RO.contains(virt_addr) {
            Region::ImageRo
        } else if IMAGE_RW.contains(virt_addr) {
            Region::ImageRw
        } else if HEAP.contains(virt_addr) {
            Region::Heap
        } else if STACK.contains(virt_addr) {
            Region::Stack
        } else if FRAME_ALLOC.contains(virt_addr) {
            Region::Physical
        } else {
            Region::Dynamic
        }
    }
}
