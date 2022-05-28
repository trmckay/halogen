#[cfg(not(test))]
use alloc::vec::Vec;

use crate::{
    align_up,
    mem::{Address, Segment},
};

#[derive(Clone, Debug)]
pub struct SegmentAllocator<T: Address> {
    segment: Segment<T>,
    allocations: Vec<Segment<T>>,
    align: usize,
    _phantom: core::marker::PhantomData<T>,
}

impl<T: Address> SegmentAllocator<T> {
    pub fn new(segment: Segment<T>, align: usize) -> SegmentAllocator<T> {
        SegmentAllocator {
            segment: segment.align_up(align),
            align,
            allocations: Vec::new(),
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn alloc(&mut self, size: usize) -> Option<T> {
        let size = align_up!(size, self.align);

        // Fast path: check if the segment fits at the end
        let tail = match self.allocations.last() {
            Some(last) => last.end,
            None => self.segment.start,
        };

        if self.segment.contains(tail + size) {
            let segment = Segment::from_size(tail, size);
            self.allocations.push(segment);
            return Some(segment.start);
        }

        // Find the index and start address if the new segment can fit between existing
        // segments
        match self
            .allocations
            .windows(2)
            .enumerate()
            .find(|(_, w)| w[1].start - w[0].end >= size)
            .map(|(i, w)| (i + 1, w[0].end))
        {
            // It fits between existing allocations
            Some((i, addr)) => {
                let segment = Segment::from_size(addr, size);
                self.allocations.insert(i, segment);
                Some(segment.start)
            }

            // Put the new allocation at the end
            None => {
                let start = match self.allocations.last() {
                    Some(last) => last.end,
                    None => self.segment.start,
                };

                let segment = Segment::from_size(start, size);

                if self.segment.contains_other(segment) {
                    self.allocations.push(segment);
                    Some(segment.start)
                } else {
                    None
                }
            }
        }
    }

    pub fn free(&mut self, addr: T) {
        self.allocations.retain(|segment| !segment.contains(addr));
    }
}
