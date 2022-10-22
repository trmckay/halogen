//! This module implements paging and memory mapping for Sv39. The main
//! interface is the `map()` function, which provides the ability to allocate
//! virtual memory, back virtual regions with physical frames, or map
//! physical regions into virtual space. The exact function depends on which, if
//! any, addresses are provided.

use halogen_common::{
    align_up, mask_range,
    mem::{Address, PhysicalAddress, Segment, VirtualAddress, GIB, KIB, MIB},
};
use spin::Mutex;

use super::{phys, regions::virtual_offset};
use crate::{
    error::{KernelError, KernelResult},
    kerror,
    mem::virt_alloc::virt_addr_alloc,
};

/// Sv39 requires satp.mode = 8.
pub const SATP_MODE: usize = 8;

/// Address-space ID for the kernel space.
pub const KERNEL_ASID: u16 = 0;

/// Level 0 page = 4K.
pub const PAGE_SIZE: usize = 4 * KIB;
pub const PAGE_MASK: usize = usize::MAX & !(PAGE_SIZE - 1);

/// Level 1 page = 2M.
pub const MEGAPAGE_SIZE: usize = 2 * MIB;
pub const MEGAPAGE_MASK: usize = usize::MAX & !(MEGAPAGE_SIZE - 1);

/// Level 2 page = 1G.
pub const GIGAPAGE_SIZE: usize = GIB;
pub const GIGAPAGE_MASK: usize = usize::MAX & !(GIGAPAGE_SIZE - 1);

const PTE_SIZE: usize = 8;
const PT_LENGTH: usize = 512;

const FIELD_VPN: usize = 0x1FF;

/// Some language features rely on position-dependent code. In practice, this
/// means nothing that uses or calls something that uses dynamic-dispatch. For
/// this kernel, `Mutex` and `print!` are the most notable cases. This flag
/// determines if those features can be used.
pub static mut PAGING_ENABLED: bool = false;

static ROOT_PAGE_TABLE_MUTEX: Mutex<()> = Mutex::new(());
static mut ROOT_PAGE_TABLE: PageTable = PageTable([PageTableEntry(0); PT_LENGTH]);

mod flags {
    /// A global mapping is used in all address-spaces.
    pub const GLOBAL: usize = 0b0010_0000;
    /// Available from user mode.
    pub const USER: usize = 0b0001_0000;
    /// Can be fetched/executed.
    pub const EXECUTE: usize = 0b0000_1000;
    /// Can be written to.
    pub const WRITE: usize = 0b0000_0100;
    /// Can be read from (and executed if MXR is set).
    pub const READ: usize = 0b0000_0010;
    /// The mapping is valid and will be dereferenced by the MMU.
    pub const VALID: usize = 0b0000_0001;
    /// For use by the kernel.
    pub const DIRTY: usize = 0b1000_0000;
    /// For use by the kernel.
    pub const ACCESSED: usize = 0b0100_0000;
}

pub fn get_root_satp() -> usize {
    get_satp(KERNEL_ASID, unsafe { &ROOT_PAGE_TABLE })
}

/// Calculate the satp register based of the mode, ASID, and root page table.
#[inline]
pub fn get_satp(asid: u16, root: &PageTable) -> usize {
    let phys_addr = if unsafe { PAGING_ENABLED } {
        let (phys_addr, _, _, _) =
            translate(VirtualAddress::from_ref(root)).expect("invalid reference");
        phys_addr
    } else {
        PhysicalAddress::from_ref(root)
    };

    (SATP_MODE << 60) | ((asid as usize) << 44) | (usize::from(phys_addr) >> 12)
}

/// Describes in which address spaces a mapping is accessible. Global mappings
/// should be mapped in all page tables (used for the kernel half).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Global,
    Local,
}

impl From<Scope> for usize {
    fn from(scope: Scope) -> usize {
        match scope {
            Scope::Global => flags::GLOBAL,
            Scope::Local => 0,
        }
    }
}

impl From<usize> for Scope {
    fn from(n: usize) -> Scope {
        match n & flags::GLOBAL {
            0 => Scope::Local,
            _ => Scope::Global,
        }
    }
}

/// Differentiate between user and kernel-only mappings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Privilege {
    Kernel,
    User,
}

impl From<Privilege> for usize {
    fn from(prv: Privilege) -> Self {
        match prv {
            Privilege::User => flags::USER,
            Privilege::Kernel => 0,
        }
    }
}

impl From<usize> for Privilege {
    fn from(n: usize) -> Privilege {
        match n & flags::USER {
            0 => Privilege::Kernel,
            _ => Privilege::User,
        }
    }
}

/// Types of address translation. A translation is either another level of
/// page tables, or a leaf physical address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Translation {
    Directory(Scope),
    Leaf(Scope, Privilege, Permissions),
}

impl From<Translation> for usize {
    fn from(meta: Translation) -> usize {
        match meta {
            Translation::Leaf(scope, prv, perms) => {
                usize::from(scope) | usize::from(prv) | usize::from(perms)
            }
            Translation::Directory(scope) => usize::from(scope),
        }
    }
}

/// Valid flag combinations.
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
        match (
            flags & flags::READ != 0,
            flags & flags::WRITE != 0,
            flags & flags::EXECUTE != 0,
        ) {
            (false, false, false)
            | (true, true, true)
            | (false, false, true)
            | (false, true, true)
            | (false, true, false) => Permissions::Invalid,
            (true, false, false) => Permissions::ReadOnly,
            (true, true, false) => Permissions::ReadWrite,
            (true, false, true) => Permissions::ReadExecute,
        }
    }
}

/// Mapping levels of Sv39. Gigapage = 1 GiB, MegaPage = 2 MiB, Page = 4 KiB.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Level {
    GigaPage,
    MegaPage,
    Page,
}

impl Level {
    /// Return the next level down, if any.
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

/// Extract the physical page numbers from a physical address.
#[inline]
fn ppn(virt_addr: VirtualAddress, level: Level) -> usize {
    let virt_addr: usize = virt_addr.into();
    match level {
        Level::GigaPage => (virt_addr & mask_range!(55, 30)) >> 30,
        Level::MegaPage => (virt_addr & mask_range!(29, 21)) >> 21,
        Level::Page => (virt_addr & mask_range!(20, 12)) >> 12,
    }
}

/// Extract the offset bits from a physical address.
#[inline]
fn offset(phys_addr: PhysicalAddress) -> usize {
    let phys_addr: usize = phys_addr.into();
    phys_addr & mask_range!(12, 0)
}

/// Extract the level-based offset as a mask to add to the address.
#[inline]
fn offset_mask(virt_addr: VirtualAddress, level: Level) -> usize {
    let virt_addr: usize = virt_addr.into();
    match level {
        Level::GigaPage => virt_addr & mask_range!(29, 0),
        Level::MegaPage => virt_addr & mask_range!(20, 0),
        Level::Page => virt_addr & mask_range!(11, 0),
    }
}

/// Extract the VPN from a virtual address.
#[inline]
fn vpn(virt_addr: VirtualAddress, level: Level) -> usize {
    let virt_addr: usize = virt_addr.into();
    match level {
        Level::GigaPage => (virt_addr >> 30) & FIELD_VPN,
        Level::MegaPage => (virt_addr >> 21) & FIELD_VPN,
        Level::Page => (virt_addr >> 12) & FIELD_VPN,
    }
}

/// A 64-bit Sv39 page table entry. Points to the next-level page table or a
/// physical address.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    /// Return the flag bits of an entry.
    fn flags(&self) -> usize {
        self.0 & 0xFF
    }

    /// Get the permissions from the flags of an entry.
    fn permissions(&self) -> Permissions {
        self.flags().into()
    }

    /// Get the scope from the flags of an entry.
    fn scope(&self) -> Scope {
        self.flags().into()
    }

    /// Get user/kernel accessibility from the flags of an entry.
    fn privilege(&self) -> Privilege {
        self.flags().into()
    }

    /// Set the flags of an entry.
    fn set_flags(&mut self, flags: usize) {
        debug_assert!(flags <= 0xFF);
        self.0 = (self.0 & !0xFF) | (flags & 0xFF);
    }

    /// Set the physical address of a leaf.
    fn set_translation(&mut self, phys_addr: PhysicalAddress, meta: Translation) {
        let flags = usize::from(meta) | flags::VALID;
        self.0 = (usize::from(phys_addr) >> 2) | (flags & 0xFF);
    }

    /// Returns true if the entry is valid.
    fn is_valid(&self) -> bool {
        self.0 & flags::VALID != 0
    }

    /// Returns true if the entry is a leaf.
    fn is_leaf(&self) -> bool {
        self.0 & (flags::READ | flags::WRITE | flags::EXECUTE) != 0
    }

    /// Extract the physical page numbers from a PTE as a mask to
    /// combine w/ the VPN portion of the translated address.
    fn page_number(&self, level: Level) -> usize {
        (match level {
            Level::GigaPage => self.0 & mask_range!(53, 28),
            Level::MegaPage => self.0 & mask_range!(53, 19),
            Level::Page => self.0 & mask_range!(53, 10),
        } << 2) as usize
    }

    /// Get a reference to the next level page table.
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

/// 512 PTE x 64-bit page table.
#[derive(Debug, Clone, Copy)]
#[repr(C, align(4096))]
pub struct PageTable([PageTableEntry; PT_LENGTH]);

impl Default for PageTable {
    fn default() -> PageTable {
        PageTable([PageTableEntry(0); PT_LENGTH])
    }
}

impl PageTable {
    /// Set all the entries to invalid.
    pub fn clear(&mut self) {
        for entry in self.0.iter_mut() {
            *entry = PageTableEntry(0);
        }
    }

    pub fn from_kernel_root() -> PageTable {
        unsafe { ROOT_PAGE_TABLE }
    }

    /// Allocate a physical page on the kernel heap and clear it for use as
    /// a page table.
    pub fn new_static() -> KernelResult<(&'static mut PageTable, PhysicalAddress)> {
        unsafe {
            let (virt_addr, phys_addr) = phys::alloc().ok_or_else(|| {
                kerror!(
                    KernelError::PageTableAllocation,
                    kerror!(KernelError::OutOfPhysicalFrames)
                )
            })?;

            let pt = (if PAGING_ENABLED {
                virt_addr.as_mut_ptr()
            } else {
                phys_addr.as_mut_ptr()
            } as *mut PageTable)
                .as_mut()
                .expect("Physical frame is null pointer");

            pt.clear();
            Ok((pt, phys_addr))
        }
    }

    pub fn entries(&self) -> &[PageTableEntry; PT_LENGTH] {
        &self.0
    }

    /// Get the `n`th entry in the page table.
    pub fn get(&self, n: usize) -> &'static mut PageTableEntry {
        unsafe {
            (core::ptr::addr_of!(self.0[n]) as *mut PageTableEntry)
                .as_mut()
                .expect("null pointer")
        }
    }

    /// Get or create the next level page table.
    pub fn get_create_next(&self, n: usize, scope: Scope) -> KernelResult<&'static mut PageTable> {
        let entry = self.get(n);
        if !entry.is_valid() {
            match PageTable::new_static() {
                Ok((pt, phys_addr)) => {
                    entry.set_translation(phys_addr, Translation::Directory(scope));
                    Ok(pt)
                }
                Err(why) => Err(why),
            }
        } else {
            match entry.next_level() {
                Some(pt) => Ok(pt),
                None => kerror!(KernelError::PageTableCorruption).into(),
            }
        }
    }

    /// Map a virtual address to a physical address.
    pub fn map(
        &mut self,
        virt_addr: VirtualAddress,
        phys_addr: PhysicalAddress,
        level: Level,
        perms: Permissions,
        scope: Scope,
        prv: Privilege,
    ) -> KernelResult<()> {
        let mut curr_level = Level::GigaPage;
        let mut pt = self;

        loop {
            // Should we continue descending?
            match (curr_level == level, curr_level.next()) {
                // Current level does not match the desired level and there are still more levels.
                (false, Some(next_level)) => {
                    // We can, so get the next level PT.
                    pt = pt.get_create_next(vpn(virt_addr, curr_level), scope)?;

                    curr_level = next_level
                }
                // Current level matches the desired level or there are no more levels.
                (true, _) | (_, None) => {
                    // We are at the leaf PT; get the entry and set the address.
                    pt.get(vpn(virt_addr, curr_level))
                        .set_translation(phys_addr, Translation::Leaf(scope, prv, perms));

                    return Ok(());
                }
            }
        }
    }

    /// Translate a virtual address to its physical address + permissions.
    pub fn translate(
        &self,
        virt_addr: VirtualAddress,
    ) -> Option<(PhysicalAddress, Scope, Privilege, Permissions)> {
        let mut pt = self;
        let mut level = Level::GigaPage;

        loop {
            let entry = pt.get(vpn(virt_addr, level));

            if entry.is_valid() {
                if entry.is_leaf() {
                    return Some((
                        PhysicalAddress(entry.page_number(level) | offset_mask(virt_addr, level)),
                        entry.scope(),
                        entry.privilege(),
                        entry.permissions(),
                    ));
                } else {
                    pt = entry.next_level()?;
                    level = level.next()?;
                }
            } else {
                return None;
            }
        }
    }

    /// Free a table, but do not free any sub-tables.
    ///
    /// # Safety
    ///
    /// - If sub-tables are only referred to by this table, they will be leaked.
    /// - Assumes that the frame allocator uses physical addresses.
    pub unsafe fn free(&mut self) {
        let (phys_addr, _, _, _) = self
            .translate(VirtualAddress(core::ptr::addr_of!(self) as usize))
            .expect("failed to get physical address of page table");

        phys::free(phys_addr)
    }

    /// Recursively free a page table and all of its sub-tables.
    ///
    /// # Safety
    ///
    /// * If other page tables refer to this one, freeing can cause page-faults.
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
/// - This can be very destructive if invalid virtual and/or physical addresses
///   are provided.
/// - If `scope` is `Global`, this mapping must be present in all page tables.
pub unsafe fn map(
    virt_base: Option<VirtualAddress>,
    phys_base: Option<PhysicalAddress>,
    size: usize,
    perms: Permissions,
    scope: Scope,
    prv: Privilege,
) -> KernelResult<VirtualAddress> {
    let size = align_up!(size, PAGE_SIZE);

    let virt_base = match virt_base {
        Some(addr) => addr,
        None => {
            virt_addr_alloc(PAGE_SIZE).ok_or_else(|| kerror!(KernelError::OutOfVirtualAddresses))?
        }
    };

    let _lock;
    if PAGING_ENABLED {
        _lock = ROOT_PAGE_TABLE_MUTEX.lock();
    }

    for offset in (0..size).step_by(PAGE_SIZE) {
        let virt_addr = virt_base + offset;
        let phys_addr = match phys_base {
            Some(phys_addr) => phys_addr + offset,
            None => {
                let (_, phys_frame) =
                    phys::alloc().ok_or_else(|| kerror!(KernelError::OutOfPhysicalFrames))?;
                phys_frame
            }
        };

        ROOT_PAGE_TABLE.map(virt_addr, phys_addr, Level::Page, perms, scope, prv)?;
    }

    Ok(virt_base)
}

/// Unmap a virtual address.
///
/// # Safety
///
/// - Unmapped memory must be unused.
pub unsafe fn unmap(segment: Segment<VirtualAddress>) -> KernelResult<()> {
    let _lock;
    if PAGING_ENABLED {
        _lock = ROOT_PAGE_TABLE_MUTEX.lock();
    }

    for virt_addr in segment.iter().step_by(PAGE_SIZE) {
        ROOT_PAGE_TABLE.map(
            virt_addr.into(),
            PhysicalAddress::null(),
            Level::Page,
            Permissions::Invalid,
            Scope::Local,
            Privilege::Kernel,
        )?;
    }

    Ok(())
}

/// Translate a virtual address to its physical address + permissions.
pub fn translate(
    virt_addr: VirtualAddress,
) -> Option<(PhysicalAddress, Scope, Privilege, Permissions)> {
    unsafe {
        let _lock;
        if PAGING_ENABLED {
            _lock = ROOT_PAGE_TABLE_MUTEX.lock();
        }
        ROOT_PAGE_TABLE.translate(virt_addr)
    }
}
