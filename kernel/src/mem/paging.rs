use crate::{is_mapping_aligned, mask_range, mem::frame_alloc, prelude::*};

/// Level 0 page = 4K
pub const L0_PAGE_SIZE: usize = 4096;
pub const L0_PAGE_MASK: usize = 0xFFFF_FFFF_FFFF_F000;

/// Level 1 page = 2M
pub const L1_PAGE_SIZE: usize = 2 * 1024 * 1024;

/// Level 2 page = 1G
pub const L2_PAGE_SIZE: usize = 1024 * 1024 * 1024;

pub const PTE_SIZE_BYTES: usize = 8;
pub const PT_LENGTH: usize = 512;

const FIELD_VPN: usize = 0x1FF;

pub static mut ROOT_PAGE_TABLE_RAW: PageTable = PageTable {
    entries: [PageTableEntry { entry: 0 }; PT_LENGTH],
};

lazy_static! {
    pub static ref ROOT_PAGE_TABLE: Mutex<&'static mut PageTable> =
        Mutex::new(unsafe { &mut ROOT_PAGE_TABLE_RAW });
    pub static ref PAGE_OFFSET: Mutex<usize> = Mutex::new(0);
}

pub mod pte_flags {
    pub const GLOBAL: usize = 0b0010_0000;
    pub const USER: usize = 0b0001_0000;
    pub const EXECUTE: usize = 0b0000_1000;
    pub const WRITE: usize = 0b0000_0100;
    pub const READ: usize = 0b0000_0010;
    pub const VALID: usize = 0b0000_0001;
    pub const DIRTY: usize = 0b1000_0000;
    pub const ACCESSED: usize = 0b0100_0000;
}

use pte_flags::*;

/// Map a virtual address to a physical address
///
/// # Safety
///
/// This maps memory into the kernel address space, so should not overwrite any
/// kernel text or structures
pub unsafe fn map(virt_addr: usize, phys_addr: usize, flags: usize) {
    ROOT_PAGE_TABLE
        .lock()
        .map(virt_addr, phys_addr, MappingLevel::FourKilobyte, flags)
}

pub fn translate(virt_addr: usize) -> Option<usize> {
    ROOT_PAGE_TABLE.lock().translate(virt_addr)
}

#[derive(Debug, Copy, Clone)]
#[repr(usize)]
pub enum MappingLevel {
    OneGigabyte = 2,
    TwoMegabyte = 1,
    FourKilobyte = 0,
}

impl From<usize> for MappingLevel {
    fn from(n: usize) -> Self {
        match n {
            0 => MappingLevel::FourKilobyte,
            1 => MappingLevel::TwoMegabyte,
            _ => MappingLevel::OneGigabyte,
        }
    }
}

/// 54-bit physical address
pub struct PhysicalAddress {
    addr: usize,
}

impl PhysicalAddress {
    /// Traverse the page-table and translate a virtual address to a physical
    /// address
    pub fn from_virt(virt_addr: VirtualAddress, root: &PageTable) -> Option<PhysicalAddress> {
        root.translate(virt_addr.addr)
            .map(PhysicalAddress::from_usize)
    }

    /// Create a physical address directly
    pub fn from_usize(addr: usize) -> PhysicalAddress {
        PhysicalAddress { addr }
    }

    /// Extract the physical page numbers from a physical address
    pub fn ppn(&self, level: usize) -> usize {
        match level {
            2 => (self.addr & mask_range!(usize, 55, 30)) >> 30,
            1 => (self.addr & mask_range!(usize, 29, 21)) >> 21,
            0 => (self.addr & mask_range!(usize, 20, 12)) >> 12,
            _ => unreachable!(),
        }
    }

    /// Extract the page offset from a physical address
    pub fn offset(&self) -> usize {
        self.addr & mask_range!(usize, 12, 0)
    }
}

/// Sv39 39-bit virtual address
pub struct VirtualAddress {
    addr: usize,
}

impl VirtualAddress {
    /// Create a virtual address directly
    pub fn from_usize(addr: usize) -> VirtualAddress {
        VirtualAddress { addr }
    }

    /// Extract the virtual page numbers from a virtual address
    pub fn vpn(&self, level: usize) -> usize {
        match level {
            2 => (self.addr >> 30) & FIELD_VPN,
            1 => (self.addr >> 21) & FIELD_VPN,
            0 => (self.addr >> 12) & FIELD_VPN,
            _ => unreachable!(),
        }
    }

    /// Extract the physical page numbers from a physical address as a mask
    /// to combine with the PPN portion
    pub fn offset(&self, level: MappingLevel) -> usize {
        match level {
            MappingLevel::OneGigabyte => self.addr & mask_range!(usize, 29, 0),
            MappingLevel::TwoMegabyte => self.addr & mask_range!(usize, 20, 0),
            MappingLevel::FourKilobyte => self.addr & mask_range!(usize, 11, 0),
        }
    }
}

/// A 64-bit Sv39 page-table entry
///
/// Points to the next-level page-table or a physical address
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct PageTableEntry {
    pub entry: usize,
}

impl PageTableEntry {
    /// Set the flags of a page-table entry
    fn set_flags(&mut self, flags: usize) {
        debug_assert!(flags <= 0xFF);
        self.entry = (self.entry & !0xFF) | (flags & 0xFF);
    }

    fn set(&mut self, ppn: usize, flags: usize) {
        debug_assert!(flags <= 0xFF);
        self.entry = (ppn >> 2) | (flags & 0xFF);
    }

    /// Set the physical address of a leaf PTE
    pub fn set_phys_addr(&mut self, phys_addr: usize, flags: usize) {
        debug_assert_ne!(0, flags & (READ | WRITE | EXECUTE));
        self.set(phys_addr as usize, flags);
    }

    /// Set the pointer to the next level page-table
    pub fn set_next_level(&mut self, next: &PageTable) {
        self.set(ptr::addr_of!(*next) as usize, VALID);
    }

    /// Returns true if the entry is valid
    pub fn is_valid(&self) -> bool {
        self.entry & VALID != 0
    }

    /// Returns true if the entry is a leaf
    pub fn is_leaf(&self) -> bool {
        self.entry & (READ | WRITE | EXECUTE) != 0
    }

    /// Extract the physical page numbers from a PTE as a mask to
    /// combine w/ the VPN portion of the translated address
    pub fn translation(&self, level: MappingLevel) -> usize {
        (match level {
            MappingLevel::OneGigabyte => self.entry & mask_range!(usize, 53, 28),
            MappingLevel::TwoMegabyte => self.entry & mask_range!(usize, 53, 19),
            MappingLevel::FourKilobyte => self.entry & mask_range!(usize, 53, 10),
        } << 2) as usize
    }

    /// Get a reference to the next level page-table
    pub fn next_level(&self) -> &'static mut PageTable {
        unsafe {
            (((self.entry & mask_range!(usize, 53, 10)) << 2) as *mut PageTable)
                .as_mut()
                .expect("next level of empty PTE")
        }
    }
}

/// 512 PTE x 64-bit page-table
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; PT_LENGTH],
}

impl PageTable {
    /// Set all the entries to invalid
    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = PageTableEntry { entry: 0 };
        }
    }

    /// Allocate a physical page on the kernel heap and clear it for use as
    /// a page-table
    pub fn new<F>(alloc: F) -> &'static mut PageTable
    where
        F: Fn() -> Option<*mut u8>, {
        let page = alloc().expect("could not allocate page");
        let pt = unsafe { PageTable::from_ptr(page as *mut PageTable) };
        pt.clear();
        pt
    }

    /// Get the `n`th entry in the page-table
    pub fn get(&self, n: usize) -> &'static mut PageTableEntry {
        unsafe {
            (ptr::addr_of!(self.entries[n]) as *mut PageTableEntry)
                .as_mut()
                .expect("null pointer")
        }
    }

    /// Get or create the next level page table
    pub fn next_level_or_alloc<F>(&self, n: usize, alloc: F) -> &'static mut PageTable
    where
        F: Fn() -> Option<*mut u8>, {
        let entry = self.get(n);
        if !entry.is_valid() {
            let pt = PageTable::new(&alloc);
            entry.set_next_level(pt);
            pt
        } else {
            entry.next_level()
        }
    }

    /// Cast a pointer to a root page-table; useful when initializing the MMU
    ///
    /// # Safety
    ///
    /// `ptr` must point to a valid page table
    pub unsafe fn from_ptr(ptr: *mut PageTable) -> &'static mut PageTable {
        ptr.as_mut().expect("null pointer")
    }

    /// Create a new MMU mapping, but specify the allocator to use for new
    /// pages
    pub fn map_with_allocator<F>(
        &mut self,
        virt_addr: usize,
        phys_addr: usize,
        level: MappingLevel,
        flags: usize,
        alloc: F,
    ) where
        F: Fn() -> Option<*mut u8>, {
        debug_assert!(is_mapping_aligned!(virt_addr, level));
        debug_assert!(is_mapping_aligned!(phys_addr, level));

        let virt_addr = VirtualAddress::from_usize(virt_addr);

        let mut vpn_num = 2;
        // Start at the root page-table
        let mut pt = self;

        // For level 1 and level 0 mappings, get the next level page-table
        if let MappingLevel::TwoMegabyte | MappingLevel::FourKilobyte = level {
            pt = pt.next_level_or_alloc(virt_addr.vpn(vpn_num), &alloc);
            vpn_num -= 1;
        }

        // For level 1 mappings, this PT contains the branch entry
        // For level 0 mapping, get the next level page table
        if let MappingLevel::FourKilobyte = level {
            pt = pt.next_level_or_alloc(virt_addr.vpn(vpn_num), &alloc);
            vpn_num -= 1;
        }

        // Get the branch entry and set the address
        pt.get(virt_addr.vpn(vpn_num))
            .set_phys_addr(phys_addr, flags);
    }

    /// Map a virtual address to a physical address
    pub fn map(&mut self, virt_addr: usize, phys_addr: usize, level: MappingLevel, flags: usize) {
        self.map_with_allocator(virt_addr, phys_addr, level, flags, frame_alloc);
    }

    /// Translate a virtual address to its physical address
    pub fn translate(&self, virt_addr: usize) -> Option<usize> {
        let virt_addr = VirtualAddress::from_usize(virt_addr);
        let mut pt = self;

        for level in (0..=2).rev() {
            let entry = pt.get(virt_addr.vpn(level));
            if entry.is_valid() {
                if entry.is_leaf() {
                    return Some(entry.translation(level.into()) | virt_addr.offset(level.into()));
                } else {
                    pt = entry.next_level();
                }
            } else {
                return None;
            }
        }

        None
    }
}
