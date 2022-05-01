use core::fmt;

#[derive(Clone, Copy, Debug)]
pub struct AddressRange {
    pub start: usize,
    pub size: usize,
}

impl AddressRange {
    pub fn new(start: usize, size: usize) -> AddressRange {
        AddressRange { start, size }
    }

    #[inline(always)]
    pub fn end(&self) -> usize {
        self.start + self.size
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> {
        self.start..(self.start + self.size)
    }

    pub fn contains(&self, addr: usize) -> bool {
        addr < self.start + self.size
    }

    pub fn contains_range(&self, other: AddressRange) -> bool {
        self.start <= other.start && self.end() >= other.end()
    }
}

impl fmt::Display for AddressRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:p}..{:p}",
            self.start as *const u8,
            self.end() as *const u8
        )
    }
}
