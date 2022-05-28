use core::slice::{from_raw_parts, from_raw_parts_mut};

use halogen_macros::Address;

use crate::align_up;

/// Address trait allows for different types of addresses to be used as
/// type-bounds on functions that accept and return addresses
///
/// This allows for the construction of types that are different to the type
/// checker, but behave as naked `usize`s in practice
pub trait Address:
    Copy
    + core::ops::Add<usize, Output = Self>
    + core::ops::Sub<Self, Output = usize>
    + core::ops::Sub<usize, Output = Self>
    + From<usize>
    + Into<usize> {
    fn as_ptr<T>(self) -> *const T;

    fn as_mut_ptr<T>(self) -> *mut T {
        self.as_ptr::<T>() as *mut T
    }

    fn is_aligned(self, to: usize) -> bool {
        self.into() % to == 0
    }

    fn offset_from<O: Address>(self, rhs: O) -> isize {
        let this: usize = self.into();
        let rhs: usize = rhs.into();

        this.wrapping_sub(rhs) as isize
    }

    fn add_offset(self, offset: isize) -> Self {
        let base: usize = self.into();
        Self::from(base.wrapping_add(offset as usize))
    }

    fn from_ptr<T>(ptr: *const T) -> Self {
        Self::from(ptr as usize)
    }

    fn is_null(self) -> bool {
        (self.as_ptr() as *const u8).is_null()
    }
}

impl Address for usize {
    fn as_ptr<T>(self) -> *const T {
        self as *const T
    }
}

/// A virtual address (39-bits, fits in a `usize`)
#[derive(Copy, Clone, Address)]
pub struct VirtualAddress(pub usize);

impl VirtualAddress {
    pub fn null() -> VirtualAddress {
        VirtualAddress(0)
    }

    pub const fn as_phys(self) -> PhysicalAddress {
        PhysicalAddress(self.0)
    }
}


/// A physical address (54-bits, fits in a `usize`)
#[derive(Copy, Clone, Address)]
pub struct PhysicalAddress(pub usize);

impl PhysicalAddress {
    pub fn null() -> PhysicalAddress {
        PhysicalAddress(0)
    }

    pub const fn as_virt(self) -> VirtualAddress {
        VirtualAddress(self.0)
    }
}

/// This is essentially `core::ops::Range<T>` for addresses; `Range` has a few
/// issues that make it inconvenient to store
#[derive(Copy, Clone, Debug)]
pub struct Segment<T: Address> {
    pub start: T,
    pub end: T,
}

impl<T: Address> Segment<T> {
    pub const fn new(start: T, end: T) -> Segment<T> {
        Segment { start, end }
    }

    pub fn from_size(start: T, size: usize) -> Segment<T> {
        Segment {
            start,
            end: start + size,
        }
    }

    /// Convert the segment to a slice.
    ///
    /// # Safety
    ///
    /// - The memory referenced by `self` must be valid.
    pub unsafe fn as_slice(&self) -> &'static [u8] {
        from_raw_parts(self.start.as_ptr(), self.size())
    }

    /// Convert the segment to a mutable slice
    ///
    /// # Safety
    ///
    /// - The memory referenced by `self` must be valid.
    /// - Can be used to generate multiple mutable references.
    pub unsafe fn as_mut_slice(&self) -> &'static mut [u8] {
        from_raw_parts_mut(self.start.as_mut_ptr(), self.size())
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    pub fn contains<I: Into<T>>(&self, other: I) -> bool {
        let other: T = other.into();
        self.start.into() <= other.into() && self.end.into() > other.into()
    }

    #[inline]
    pub fn is_aligned(&self, to: usize) -> bool {
        self.start.into() % to == 0 && self.end.into() % to == 0
    }

    #[inline]
    pub fn contains_other(&self, other: Segment<T>) -> bool {
        self.start.into() <= other.start.into() && self.end.into() >= other.end.into()
    }

    #[inline]
    pub fn shift(self, offset: isize) -> Segment<T> {
        let start: usize = self.start.into();
        let end: usize = self.end.into();
        Segment {
            start: T::from(start.wrapping_add(offset as usize)),
            end: T::from(end.wrapping_add(offset as usize)),
        }
    }

    #[inline]
    pub fn truncate(self, size: usize) -> Segment<T> {
        if size < self.size() {
            let diff = self.size() - size;
            Segment {
                start: self.start,
                end: self.end - diff,
            }
        } else {
            self
        }
    }

    #[inline]
    pub fn align_up(self, align: usize) -> Segment<T> {
        Segment {
            start: T::from(align_up!(self.start.into(), align)),
            end: self.end,
        }
    }
}

impl<T: Address> core::fmt::Display for Segment<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:p}..{:p}",
            self.start.as_ptr::<u8>(),
            self.end.as_ptr::<u8>()
        )
    }
}

impl<T: Address, I: Into<usize>> From<core::ops::Range<I>> for Segment<T> {
    fn from(range: core::ops::Range<I>) -> Segment<T> {
        Segment {
            start: T::from(range.start.into()),
            end: T::from(range.end.into()),
        }
    }
}
