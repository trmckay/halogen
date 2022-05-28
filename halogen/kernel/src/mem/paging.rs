use halogen_common::{
    align_up, mask_range,
    mem::{Address, PhysicalAddress, VirtualAddress, GIB, KIB, MIB},
};
use spin::Mutex;

use super::{phys, regions::virtual_offset};
use crate::{error::KernelError, log::*, mem::virt_alloc::virt_addr_alloc};

pub const SATP_MODE: usize = 8;

pub const KERNEL_ASID: usize = 0;

/// Level 0 page = 4K
pub const PAGE_SIZE: usize = 4 * KIB;
pub const PAGE_MASK: usize = usize::MAX & !(PAGE_SIZE - 1);

/// Level 1 page = 2M
pub const MEGAPAGE_SIZE: usize = 2 * MIB;
pub const MEGAPAGE_MASK: usize = usize::MAX & !(MEGAPAGE_SIZE - 1);

/// Level 2 page = 1G
pub const GIGAPAGE_SIZE: usize = GIB;
pub const GIGAPAGE_MASK: usize = usize::MAX & !(GIGAPAGE_SIZE - 1);

const PTE_SIZE: usize = 8;
const PT_LENGTH: usize = 512;

const FIELD_VPN: usize = 0x1FF;

/// True if paging is fully set up
///
/// Some language features rely on position-dependent code; practically, this
/// means no mutexs or trait objects
pub static mut PAGING_ENABLED: bool = false;

static ROOT_PAGE_TABLE_MUTEX: Mutex<()> = Mutex::new(());
static mut ROOT_PAGE_TABLE: PageTable = PageTable([PageTableEntry(0); PT_LENGTH]);

mod flags {
    pub const GLOBAL: usize = 0b0010_0000;
    pub const USER: usize = 0b0001_0000;
    pub const EXECUTE: usize = 0b0000_1000;
    pub const WRITE: usize = 0b0000_0100;
    pub const READ: usize = 0b0000_0010;
    pub const VALID: usize = 0b0000_0001;
    pub const DIRTY: usize = 0b1000_0000;
    pub const ACCESSED: usize = 0b0100_0000;
}

#[inline]
pub fn root_satp() -> usize {
    unsafe {
        (SATP_MODE << 60)
            | (KERNEL_ASID << 44)
            | ((core::ptr::addr_of!(ROOT_PAGE_TABLE) as usize) >> 12)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Translation {
    Directory,
    Leaf(Permissions),
}

impl From<Translation> for usize {
    fn from(meta: Translation) -> usize {
        match meta {
            Translation::Leaf(perm) => perm.into(),
            Translation::Directory => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permissions {
    ReadOnly,
    ReadExecute,
    ReadWrite,
    Invalid,
}

impl From<Permissions> for usize {
    fn from(perm: Permissions) -> usize {
        match perm {
            Permissions::ReadOnly => flags::READ,
            Permissions::ReadExecute => flags::READ | flags::EXECUTE,
            Permissions::ReadWrite => flags::READ | flags::WRITE,
            Permissions::Invalid => 0,
        }
    }
}

impl From<usize> for Permissions {
    fn from(flags: usize) -> Permissions {
        let flags = flags & !flags::VALID;
        if flags & !flags::READ == 0 {
            Permissions::ReadOnly
        } else if flags & !(flags::READ | flags::EXECUTE) == 0 {
            Permissions::ReadExecute
        } else if flags & !(flags::READ | flags::WRITE) == 0 {
            Permissions::ReadWrite
        } else {
            Permissions::Invalid
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Level {
    GigaPage,
    MegaPage,
    Page,
}

impl Level {
    pub fn next(&self) -> Option<Level> {
        match &self {
            Level::GigaPage => Some(Level::MegaPage),
            Level::MegaPage => Some(Level::Page),
            Level::Page => None,
        }
    }
}

impl From<Level> for usize {
    fn from(level: Level) -> usize {
        match level {
            Level::GigaPage => 0,
            Level::MegaPage => 1,
            Level::Page => 2,
        }
    }
}

/// Extract the physical page numbers from a physical address
#[inline]
fn ppn(virt_addr: VirtualAddress, level: Level) -> usize {
    let virt_addr: usize = virt_addr.into();
    match level {
        Level::GigaPage => (virt_addr & mask_range!(55, 30)) >> 30,
        Level::MegaPage => (virt_addr & mask_range!(29, 21)) >> 21,
        Level::Page => (virt_addr & mask_range!(20, 12)) >> 12,
    }
}

/// Extract the offset bits from a physical address
#[inline]
fn offset(phys_addr: PhysicalAddress) -> usize {
    let phys_addr: usize = phys_addr.into();
    phys_addr & mask_range!(12, 0)
}

/// Extract the level-based offset as a mask to add to the address
#[inline]
fn offset_mask(virt_addr: VirtualAddress, level: Level) -> usize {
    let virt_addr: usize = virt_addr.into();
    match level {
        Level::GigaPage => virt_addr & mask_range!(29, 0),
        Level::MegaPage => virt_addr & mask_range!(20, 0),
        Level::Page => virt_addr & mask_range!(11, 0),
    }
}

/// Extract the VPN from a virtual address
#[inline]
fn vpn(virt_addr: VirtualAddress, level: Level) -> usize {
    let virt_addr: usize = virt_addr.into();
    match level {
        Level::GigaPage => (virt_addr >> 30) & FIELD_VPN,
        Level::MegaPage => (virt_addr >> 21) & FIELD_VPN,
        Level::Page => (virt_addr >> 12) & FIELD_VPN,
    }
}

/// A 64-bit Sv39 page-table entry
///
/// Points to the next-level page-table or a physical address
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    fn flags(&self) -> usize {
        self.0 & 0xFF
    }

    /// Set the flags of a page-table entry
    fn set_flags(&mut self, flags: usize) {
        debug_assert!(flags <= 0xFF);
        self.0 = (self.0 & !0xFF) | (flags & 0xFF);
    }

    /// Set the physical address of a leaf PTE
    fn set_translation(&mut self, phys_addr: PhysicalAddress, meta: Translation) {
        let flags = usize::from(meta) | flags::VALID;
        self.0 = (usize::from(phys_addr) >> 2) | (flags & 0xFF);
    }

    /// Returns true if the entry is valid
    fn is_valid(&self) -> bool {
        self.0 & flags::VALID != 0
    }

    /// Returns true if the entry is a leaf
    fn is_leaf(&self) -> bool {
        self.0 & (flags::READ | flags::WRITE | flags::EXECUTE) != 0
    }

    /// Extract the physical page numbers from a PTE as a mask to
    /// combine w/ the VPN portion of the translated address
    fn page_number(&self, level: Level) -> usize {
        (match level {
            Level::GigaPage => self.0 & mask_range!(53, 28),
            Level::MegaPage => self.0 & mask_range!(53, 19),
            Level::Page => self.0 & mask_range!(53, 10),
        } << 2) as usize
    }

    /// Get a reference to the next level page-table
    fn next_level(&self) -> Option<&'static mut PageTable> {
        unsafe {
            (((self.0 & mask_range!(53, 10)) << 2).wrapping_add(if PAGING_ENABLED {
                virtual_offset() as usize
            } else {
                0
            }) as *mut PageTable)
                .as_mut()
        }
    }
}

/// 512 PTE x 64-bit page-table
#[derive(Debug)]
#[repr(C, align(4096))]
pub struct PageTable([PageTableEntry; PT_LENGTH]);

impl PageTable {
    /// Set all the entries to invalid
    pub fn clear(&mut self) {
        for entry in self.0.iter_mut() {
            *entry = PageTableEntry(0);
        }
    }

    /// Allocate a physical page on the kernel heap and clear it for use as
    /// a page-table
    pub fn new() -> Option<(&'static mut PageTable, PhysicalAddress)> {
        unsafe {
            let (virt_addr, phys_addr) = phys::alloc()?;
            let pt = (if PAGING_ENABLED {
                virt_addr.as_mut_ptr()
            } else {
                phys_addr.as_mut_ptr()
            } as *mut PageTable)
                .as_mut()?;
            pt.clear();
            Some((pt, phys_addr))
        }
    }

    /// Get the `n`th entry in the page-table
    pub fn get(&self, n: usize) -> &'static mut PageTableEntry {
        unsafe {
            (core::ptr::addr_of!(self.0[n]) as *mut PageTableEntry)
                .as_mut()
                .expect("null pointer")
        }
    }

    /// Get or create the next level page table
    pub fn next_level_table(&self, n: usize) -> Option<&'static mut PageTable> {
        let entry = self.get(n);
        if !entry.is_valid() {
            match PageTable::new() {
                Some((pt, phys_addr)) => {
                    entry.set_translation(phys_addr, Translation::Directory);
                    Some(pt)
                }
                None => None,
            }
        } else {
            entry.next_level()
        }
    }

    /// Map a virtual address to a physical address
    pub fn map(
        &mut self,
        virt_addr: VirtualAddress,
        phys_addr: PhysicalAddress,
        level: Level,
        perms: Permissions,
    ) -> Result<(), KernelError> {
        let mut curr_level = Level::GigaPage;
        let mut pt = self;

        loop {
            // Should we continue descending
            match (curr_level == level, curr_level.next()) {
                // Current level does not match the desired level and there are still more levels
                (false, Some(next_level)) => {
                    // We can, so get the next level PT
                    pt = match pt.next_level_table(vpn(virt_addr, curr_level)) {
                        Some(pt) => pt,
                        None => return Err(KernelError::PageTableAllocation),
                    };

                    curr_level = next_level
                }
                // Current level matches the desired level or there are no more levels
                (true, _) | (_, None) => {
                    // We are at the leaf PT; get the entry and set the address
                    pt.get(vpn(virt_addr, curr_level))
                        .set_translation(phys_addr, Translation::Leaf(perms));

                    return Ok(());
                }
            }
        }
    }

    /// Translate a virtual address to its physical address
    pub fn translate(&self, virt_addr: VirtualAddress) -> Option<(PhysicalAddress, Permissions)> {
        let mut pt = self;
        let mut level = Level::GigaPage;

        loop {
            let entry = pt.get(vpn(virt_addr, level));

            if entry.is_valid() {
                if entry.is_leaf() {
                    return Some((
                        PhysicalAddress(entry.page_number(level) | offset_mask(virt_addr, level)),
                        entry.flags().into(),
                    ));
                } else {
                    pt = entry.next_level()?;
                    level = level.next()?;
                }
            }
        }
    }

    /// Free a table, but do not free any sub-tables
    ///
    /// # Safety
    ///
    /// * If sub-tables are only referred to by this table, they will be leaked
    /// * Assumes that the frame allocator uses physical addresses
    pub unsafe fn free(&mut self) {
        let (phys_addr, _) = self
            .translate(VirtualAddress(core::ptr::addr_of!(self) as usize))
            .expect("failed to get physical address of page table");

        phys::free(phys_addr)
    }

    /// Recursively free a page-table and all of its sub-tables
    ///
    /// # Safety
    ///
    /// * If other page tables refer to this one, freeing can cause page-faults
    pub unsafe fn recursive_free(&mut self) {
        self.0.iter_mut().for_each(|e| {
            if e.is_valid() && !e.is_leaf() {
                if let Some(table) = e.next_level() {
                    table.recursive_free();
                }
            }
        });

        self.free();
    }
}

/// Map a virtual address to a physical address. If no virtual address is
/// provided, one is chosen by the kernel. If no physical address is provided, a
/// frame is allocated. Frames are not guaranteed to be contiguous in physical
/// memory On success, returns the mapped virtual address.
///
/// This can be used to assign virtual addresses to devices, or as a
/// page-grained `vmalloc` implementation.
///
/// # Safety
///
/// * This can be very destructive if invalid virtual and/or physical addresses
///   are provided
pub unsafe fn map(
    virt_base: Option<VirtualAddress>,
    phys_base: Option<PhysicalAddress>,
    size: usize,
    perms: Permissions,
) -> Result<VirtualAddress, KernelError> {
    let size = align_up!(size, PAGE_SIZE);

    let virt_base = match virt_base {
        Some(addr) => addr,
        None => {
            match virt_addr_alloc(PAGE_SIZE) {
                Ok(virt_addr) => virt_addr,
                Err(_) => return Err(KernelError::OutOfVirtualAddresses),
            }
        }
    };

    for offset in (0..size).step_by(PAGE_SIZE) {
        let virt_addr = virt_base + offset;
        let phys_addr = match phys_base {
            Some(phys_addr) => phys_addr + offset,
            None => {
                match phys::alloc() {
                    Some((_, phys_frame)) => phys_frame,
                    None => return Err(KernelError::OutOfPhysicalFrames),
                }
            }
        };

        let _lock;
        if PAGING_ENABLED {
            _lock = ROOT_PAGE_TABLE_MUTEX.lock();
        }

        if let Err(why) = ROOT_PAGE_TABLE.map(virt_addr, phys_addr, Level::Page, perms) {
            error!("Failed to map: {:?}", why);
            return Err(why);
        }
    }

    Ok(virt_base)
}

/// Unmap a virtual address
///
/// # Safety
///
/// * Unmapped memory must be unused
pub unsafe fn unmap(virt_addr: VirtualAddress) -> Result<(), KernelError> {
    trace!("Unmap {}", virt_addr);
    todo!()
}

fn translate(virt_addr: VirtualAddress) -> Option<(PhysicalAddress, Permissions)> {
    unsafe {
        let _lock;
        if PAGING_ENABLED {
            _lock = ROOT_PAGE_TABLE_MUTEX.lock();
        }
        ROOT_PAGE_TABLE.translate(virt_addr)
    }
}
