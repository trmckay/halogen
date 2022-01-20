use core::{arch::asm, ptr::addr_of};

use crate::{
    mask_range,
    mem::{
        kmalloc,
        kmalloc::{get_bitmap, KHEAP_SIZE},
        satp_set_mode, satp_set_ppn, SatpMode,
    },
};

/// Level 0 page = 4K
pub const L0_PAGE_SIZE: usize = 4096;

/// Level 1 page = 2M
pub const L1_PAGE_SIZE: usize = 2 * 1024 * 1024;

/// Level 2 page = 1G
pub const L2_PAGE_SIZE: usize = 1024 * 1024 * 1024;

pub const PTE_SIZE_BYTES: usize = 8;
pub const PT_LENGTH: usize = 512;

pub const FLAG_GLOBAL: u64 = 0b0010_0000;
pub const FLAG_USER: u64 = 0b0001_0000;
pub const FLAG_EXECUTE: u64 = 0b0000_1000;
pub const FLAG_WRITE: u64 = 0b0000_0100;
pub const FLAG_READ: u64 = 0b0000_0010;

const FLAG_VALID: u64 = 0b0000_0001;
const FLAG_DIRTY: u64 = 0b1000_0000;
const FLAG_ACCESSED: u64 = 0b0100_0000;

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
            2 => (self.addr & mask_range!(usize, 38, 30)) >> 30,
            1 => (self.addr & mask_range!(usize, 29, 21)) >> 21,
            0 => (self.addr & mask_range!(usize, 20, 12)) >> 12,
            _ => unreachable!(),
        }
    }

    /// Extract the physical page numbers from a physical address as a mask
    /// to combine with the PPN portion
    pub fn vpn_translation(&self, level: usize) -> usize {
        match level {
            2 => self.addr & mask_range!(usize, 29, 0),
            1 => self.addr & mask_range!(usize, 20, 0),
            0 => self.addr & mask_range!(usize, 11, 0),
            _ => unreachable!(),
        }
    }

    /// Extract the page offset from a virtual address
    pub fn offset(&self) -> usize {
        self.addr & mask_range!(usize, 12, 0)
    }
}

/// A 64-bit Sv39 page-table entry; points to the next-level
/// page-table or a physical address
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct PageTableEntry {
    pub entry: u64,
}

impl PageTableEntry {
    /// Set the flags of a page-table entry
    fn set_flags(&mut self, flags: u64) {
        self.entry = (self.entry & !0xFF) | (flags & 0xFF);
    }

    /// Set a PTE's PPN (not for external use)
    fn set_pnn(&mut self, ppn: u64) {
        // Shift right 12 for 44 MSBs of address, then right 10 to place in the PTE
        // Also set the valid bit
        self.entry = (self.entry & !mask_range!(u64, 53, 10)) | (ppn >> 2) | FLAG_VALID;
    }

    /// Set the physical address of a leaf PTE
    pub fn set_phys_addr(&mut self, phys_addr: usize, flags: u64) {
        debug_assert!(flags & (FLAG_READ | FLAG_WRITE | FLAG_EXECUTE) != 0);
        self.set_pnn(phys_addr as u64);
        self.set_flags(flags | FLAG_VALID);
    }

    /// Set the pointer to the next level page-table
    pub fn set_next_level(&mut self, next: &PageTable) {
        self.set_pnn(addr_of!(*next) as u64)
    }

    /// Returns true if the entry is valid
    pub fn is_valid(&self) -> bool {
        self.entry & FLAG_VALID != 0
    }

    /// Returns true if the entry is a leaf
    pub fn is_leaf(&self) -> bool {
        self.entry & (FLAG_READ | FLAG_WRITE | FLAG_EXECUTE) != 0
    }

    /// Extract the physical page numbers from a PTE as a mask to
    /// combine w/ the VPN portion of the translated address
    pub fn ppn_translation(&self, level: usize) -> usize {
        (match level {
            2 => self.entry & mask_range!(u64, 53, 28),
            1 => self.entry & mask_range!(u64, 53, 19),
            0 => self.entry & mask_range!(u64, 53, 10),
            _ => unreachable!(),
        } << 2) as usize
    }

    /// Get a reference to the next level page-table
    pub fn next_level(&self) -> &'static mut PageTable {
        unsafe {
            (((self.entry & mask_range!(u64, 53, 10)) << 2) as *mut PageTable)
                .as_mut()
                .expect("Next level of invalid PTE")
        }
    }
}

/// 512 PTE x 64-bit page-table
#[repr(C, packed)]
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
    pub fn new() -> &'static mut PageTable {
        let alloc = kmalloc(1).expect("Could not allocate space for page-table");
        let pt = PageTable::from_ptr(alloc as *mut PageTable);
        pt.clear();
        pt
    }

    /// Get the `n`th entry in the page-table
    pub fn get(&self, n: usize) -> &'static mut PageTableEntry {
        unsafe {
            (addr_of!(self.entries[n]) as *mut PageTableEntry)
                .as_mut()
                .unwrap()
        }
    }

    /// Cast a pointer to a root page-table; useful when initializing the MMU
    pub fn from_ptr(ptr: *mut PageTable) -> &'static mut PageTable {
        unsafe {
            ptr.as_mut()
                .expect("Cannot create PageTable reference from null")
        }
    }

    /// Create a new MMU mapping
    /// TODO: support L2 and L1 mappings
    pub fn map(&mut self, virt_addr: usize, phys_addr: usize, flags: u64) {
        let phys_addr = PhysicalAddress::from_usize(phys_addr);
        let virt_addr = VirtualAddress::from_usize(virt_addr);

        let mut pt = self;
        let mut entry;

        for i in (1..=2).rev() {
            entry = pt.get(virt_addr.vpn(i));

            if !entry.is_valid() {
                let new_pt = PageTable::new();
                entry.set_next_level(new_pt);
                pt = new_pt;
            } else {
                pt = entry.next_level();
            }
        }

        pt.get(phys_addr.ppn(0))
            .set_phys_addr(phys_addr.addr, flags);
    }

    /// Translate the virtual address and return the physical address
    /// if it is mapped
    pub fn translate(&self, virt_addr: usize) -> Option<usize> {
        let virt_addr = VirtualAddress::from_usize(virt_addr);

        let mut pt = self;
        let mut entry;

        for i in (0..=2).rev() {
            let vpn = virt_addr.vpn(i);
            entry = pt.get(vpn);

            if !entry.is_valid() {
                return None;
            } else if entry.is_leaf() {
                return Some(
                    entry.ppn_translation(i) | virt_addr.vpn_translation(i) | virt_addr.offset(),
                );
            } else {
                pt = entry.next_level();
            }
        }
        None
    }
}

/// Initialize the root page-table and map the kernel
#[no_mangle]
pub unsafe extern "C" fn paging_init(kernel_start: usize, kernel_end: usize) -> ! {
    let root = PageTable::new();

    // Map the kernel text, data, and heap bitmap
    for i in (0..(kernel_end - kernel_start)).step_by(L0_PAGE_SIZE) {
        root.map(
            kernel_start + i,
            kernel_start + i,
            FLAG_READ | FLAG_WRITE | FLAG_EXECUTE,
        );
    }

    // Map the kernel heap bitmap
    let kheap_bitmap_addr = addr_of!(*get_bitmap());
    root.map(
        kheap_bitmap_addr as usize,
        kheap_bitmap_addr as usize,
        FLAG_READ | FLAG_WRITE | FLAG_EXECUTE,
    );

    // Map the kernel heap
    let kheap_addr = addr_of!(*get_bitmap()).add(L0_PAGE_SIZE);
    for i in (0..KHEAP_SIZE).step_by(L0_PAGE_SIZE) {
        root.map(
            kheap_addr as usize + i,
            kheap_addr as usize + i,
            FLAG_READ | FLAG_WRITE | FLAG_EXECUTE,
        )
    }

    // Set the MXR bit so instructions are fetched as executable
    asm!("csrc sstatus, {}", in(reg) 1 << 19);
    // Set stvec interrupt vector to kmain; this is only strictly
    // necessary when not identity mapping and the instruction
    // following enabling paging triggers a trap
    asm!("csrw stvec, {}", in(reg) crate::kmain);

    // Write Sv39 config to the satp register
    satp_set_ppn(root);
    satp_set_mode(SatpMode::Sv39);

    // Flush the TLB
    asm!("sfence.vma");
    // Trap here
    asm!("nop");
    // We didn't trap, so we must be identity mapped
    asm!("j kmain", options(noreturn));
}
