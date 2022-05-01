use crate::{mem::AddressRange, prelude::*};

#[derive(Clone, Debug)]
pub struct AddressAllocator {
    range: AddressRange,
    allocations: Vec<AddressRange>,
}

impl AddressAllocator {
    pub fn new(start: usize, size: usize) -> AddressAllocator {
        AddressAllocator {
            range: AddressRange::new(start, size),
            allocations: Vec::new(),
        }
    }

    pub fn alloc_range(&mut self, size: usize) -> Option<AddressRange> {
        // Find the index and start address if the new range can fit between existing
        // ranges
        match self
            .allocations
            .windows(2)
            .enumerate()
            .find(|(_, w)| w[1].start - (w[0].start + w[0].size) >= size)
            .map(|(i, w)| ((i + 1), w[0].start + w[0].size))
        {
            // It fits between existing allocations
            Some((i, addr)) => {
                let range = AddressRange { start: addr, size };
                self.allocations.insert(i, range);
                Some(range)
            }
            // Put the new allocation at the end
            None => {
                let end = match self.allocations.last() {
                    Some(last) => last.start + last.size,
                    None => self.range.start,
                };

                let range = AddressRange {
                    start: end + size,
                    size,
                };

                if self.range.contains_range(range) {
                    self.allocations.push(range);
                    Some(range)
                } else {
                    None
                }
            }
        }
    }

    pub fn release_range(&mut self, addr: usize) {
        self.allocations
            .retain(|alloc| !(alloc.start < addr && alloc.start + alloc.size < addr));
    }
}
