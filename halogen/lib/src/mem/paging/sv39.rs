use super::*;
use crate::{
    div_ceil, mask,
    mem::{Address, GIB, KIB, MIB},
};

/// A global mapping is used in all address-spaces.
pub const FLAG_GLOBAL: u64 = 0b0010_0000;
/// Available from user mode.
pub const FLAG_USER: u64 = 0b0001_0000;
/// Can be fetched/executed.
pub const FLAG_EXECUTE: u64 = 0b0000_1000;
/// Can be written to.
pub const FLAG_WRITE: u64 = 0b0000_0100;
/// Can be read from (and executed if MXR is set).
pub const FLAG_READ: u64 = 0b0000_0010;
/// The mapping is valid and will be dereferenced by the MMU.
pub const FLAG_VALID: u64 = 0b0000_0001;
/// For use by the kernel.
pub const FLAG_DIRTY: u64 = 0b1000_0000;
/// For use by the kernel.
pub const FLAG_ACCESSED: u64 = 0b0100_0000;

/// Three levels of a Sv39 page-table hierarchy.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Level {
    Root,
    Directory,
    Leaf,
}

impl Level {
    /// Get the size of the level.
    fn size(&self) -> usize {
        match &self {
            Level::Root => GIB,
            Level::Directory => 2 * MIB,
            Level::Leaf => 4 * KIB,
        }
    }

    // Get the next level down.
    pub fn next(&self) -> Option<Level> {
        match &self {
            Level::Root => Some(Level::Directory),
            Level::Directory => Some(Level::Leaf),
            Level::Leaf => None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Sv39Entry(u64);

impl From<LeafMapping> for Sv39Entry {
    fn from(mapping: LeafMapping) -> Sv39Entry {
        Sv39Entry(
            (usize::from(mapping.addr) as u64 >> 2)
                | Sv39Entry::owner_mask(mapping.owner)
                | Sv39Entry::permissions_mask(mapping.perms)
                | Sv39Entry::scope_mask(mapping.scope)
                | FLAG_VALID,
        )
    }
}

impl From<DirectoryMapping> for Sv39Entry {
    fn from(mapping: DirectoryMapping) -> Sv39Entry {
        Sv39Entry(
            (usize::from(mapping.next) as u64 >> 2)
                | Sv39Entry::scope_mask(mapping.scope)
                | FLAG_VALID,
        )
    }
}

impl From<Mapping> for Sv39Entry {
    fn from(mapping: Mapping) -> Self {
        match mapping {
            Mapping::Leaf(leaf) => Self::from(leaf),
            Mapping::Directory(dir) => Self::from(dir),
        }
    }
}

impl Sv39Entry {
    /// Calculate a mask to set the user bit.
    fn owner_mask(owner: Owner) -> u64 {
        match owner {
            Owner::User => FLAG_USER,
            Owner::Kernel => 0,
        }
    }

    /// Calculate a mask to set entry the permission bits.
    fn permissions_mask(permissions: Permissions) -> u64 {
        match permissions {
            Permissions::ReadOnly => FLAG_READ,
            Permissions::ReadExecute => FLAG_READ | FLAG_EXECUTE,
            Permissions::ReadWrite => FLAG_READ | FLAG_WRITE,
            Permissions::ReadWriteExecute => FLAG_READ | FLAG_WRITE | FLAG_EXECUTE,
            Permissions::Nothing | Permissions::Invalid => 0,
        }
    }

    /// Calculate a mask to set the global bit.
    fn scope_mask(scope: Scope) -> u64 {
        match scope {
            Scope::Global => FLAG_GLOBAL,
            Scope::Unique => 0,
        }
    }

    fn is_valid(self) -> bool {
        self.0 & FLAG_VALID != 0
    }

    /// Get the base of the physical page pointed to by this entry.
    fn phys_page(self, level: Level) -> PhysicalAddress {
        PhysicalAddress(
            (match level {
                Level::Root => self.0 & mask!(28, 53),
                Level::Directory => self.0 & mask!(19, 53),
                Level::Leaf => self.0 & mask!(10, 53),
            } << 2) as usize,
        )
    }

    /// Decode an entry into a leaf or directory mapping. This is not the full
    /// translation, since the remaining part of the virtual address must be
    /// added as an offset.
    fn decode(self, level: Level) -> Option<Mapping> {
        self.is_valid()
            .then_some(match Permissions::from(self) {
                // Valid bit is not set, no entry here.
                Permissions::Invalid => None,
                // Valid bit is set and no RWX is set, so it's a directory.
                Permissions::Nothing => {
                    Some(Mapping::Directory(DirectoryMapping::new(
                        // Directories use the whole physical address; nothing from the virtual
                        // address.
                        self.phys_page(Level::Leaf),
                        Scope::from(self),
                    )))
                }
                // Some permissions on a valid entry means leaf.
                perms => {
                    Some(Mapping::Leaf(LeafMapping::new(
                        self.phys_page(level),
                        perms,
                        Owner::from(self),
                        Scope::from(self),
                        level.size(),
                    )))
                }
            })
            .flatten()
    }
}

impl VirtualAddress {
    /// Extract the VPN from a virtual address. This is used as the index into
    /// the page table.
    fn vpn(self, level: Level) -> usize {
        const VPN_MASK: usize = 0x1FF;
        let va: usize = self.into();
        match level {
            Level::Root => (va >> 30) & VPN_MASK,
            Level::Directory => (va >> 21) & VPN_MASK,
            Level::Leaf => (va >> 12) & VPN_MASK,
        }
    }
}

// Decoding

impl From<Sv39Entry> for Owner {
    fn from(e: Sv39Entry) -> Owner {
        match e.0 & FLAG_USER {
            0 => Owner::Kernel,
            _ => Owner::User,
        }
    }
}

impl From<Sv39Entry> for Permissions {
    fn from(e: Sv39Entry) -> Permissions {
        match (
            e.0 & FLAG_READ != 0,
            e.0 & FLAG_WRITE != 0,
            e.0 & FLAG_EXECUTE != 0,
        ) {
            (false, false, false) => Permissions::Nothing,
            (true, true, true) => Permissions::ReadWriteExecute,
            (true, false, false) => Permissions::ReadOnly,
            (true, true, false) => Permissions::ReadWrite,
            (true, false, true) => Permissions::ReadExecute,
            (false, false, true) | (false, true, true) | (false, true, false) => {
                Permissions::Invalid
            }
        }
    }
}

impl From<Sv39Entry> for Scope {
    fn from(e: Sv39Entry) -> Scope {
        match e.0 & FLAG_GLOBAL {
            0 => Scope::Unique,
            _ => Scope::Global,
        }
    }
}

#[repr(C, align(4096))]
#[derive(Debug)]
struct Sv39Table([Sv39Entry; 512]);

impl Default for Sv39Table {
    fn default() -> Sv39Table {
        Sv39Table([Sv39Entry::default(); 512])
    }
}

type PageTableAllocator = fn() -> *mut u8;

// Implements the Sv39 paging scheme.
pub struct Sv39Manager {
    root: *mut Sv39Table,
    alloc: PageTableAllocator,
}

impl Sv39Manager {
    pub fn new(allocator: PageTableAllocator) -> Sv39Manager {
        let root = (allocator)() as *mut Sv39Table;
        Sv39Manager {
            root,
            alloc: allocator,
        }
    }

    fn map_one(
        &mut self,
        vaddr: VirtualAddress,
        paddr: PhysicalAddress,
        perms: Permissions,
        owner: Owner,
        scope: Scope,
    ) -> Result<(), PagingError> {
        let mut curr_pt = unsafe { self.root.as_mut().ok_or(PagingError::PageTableCorruption)? };

        let mut curr_level = Level::Root;

        while let Some(next_level) = curr_level.next() {
            let vpn = vaddr.vpn(curr_level);

            curr_pt = match curr_pt.0[vpn].decode(curr_level) {
                Some(Mapping::Leaf(_)) => todo!("Handle unexpected leaf"),
                Some(Mapping::Directory(d)) => unsafe {
                    d.next.as_mut().ok_or(PagingError::PageTableCorruption)?
                },
                None => {
                    let next_pt = unsafe {
                        ((self.alloc)() as *mut Sv39Table)
                            .as_mut()
                            .ok_or(PagingError::AllocationError)?
                    };
                    curr_pt.0[vpn] = Sv39Entry::from(DirectoryMapping::new(
                        PhysicalAddress::from_ptr(next_pt),
                        scope,
                    ));
                    next_pt
                }
            };

            curr_level = next_level;
        }

        curr_pt.0[vaddr.vpn(curr_level)] = Sv39Entry::from(LeafMapping::new(
            paddr,
            perms,
            owner,
            scope,
            Level::Leaf.size(),
        ));

        Ok(())
    }
}

impl PagingScheme for Sv39Manager {
    fn map(&mut self, vaddr: VirtualAddress, mapping: LeafMapping) -> Result<(), PagingError> {
        if usize::from(vaddr) % Level::Leaf.size() != 0 {
            return Err(PagingError::MisalignedVirtAddr);
        }
        if usize::from(mapping.addr) % Level::Leaf.size() != 0 {
            return Err(PagingError::MisalignedPhysAddr);
        }
        if mapping.size != Level::Leaf.size() {
            todo!("Support L0 and L1 mappings")
        }

        let count = div_ceil!(mapping.size, Level::Leaf.size());

        for _ in 0..count {
            self.map_one(
                vaddr,
                mapping.addr,
                mapping.perms,
                mapping.owner,
                mapping.scope,
            )?;
        }
        Ok(())
    }

    fn translate(&self, vaddr: VirtualAddress) -> Option<LeafMapping> {
        let mut curr_pt = unsafe { self.root.as_mut()? };

        let mut curr_level = Level::Root;

        loop {
            match curr_pt.0[vaddr.vpn(curr_level)].decode(curr_level)? {
                Mapping::Leaf(leaf) => return Some(leaf),
                Mapping::Directory(d) => curr_pt = unsafe { d.next.as_mut()? },
            }
            curr_level = curr_level.next()?;
        }
    }

    fn unmap(&mut self, _vaddr: VirtualAddress) -> Result<(), PagingError> {
        todo!("unmap not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    pub struct TestAllocator {
        allocations: Vec<Sv39Table>,
    }

    impl TestAllocator {
        fn alloc(&mut self) -> *mut u8 {
            self.allocations.push(Sv39Table::default());
            let i = self.allocations.len() - 1;
            core::ptr::addr_of_mut!(self.allocations.as_mut_slice()[i]) as *mut u8
        }
    }

    static mut TEST_ALLOCATOR: Option<TestAllocator> = None;

    fn test_alloc() -> *mut u8 {
        unsafe {
            if TEST_ALLOCATOR.is_none() {
                TEST_ALLOCATOR = Some(TestAllocator::default())
            }
        }
        unsafe { TEST_ALLOCATOR.as_mut().unwrap().alloc() }
    }

    #[test]
    fn encode_and_decode() {
        for perms in [
            Permissions::ReadOnly,
            Permissions::ReadWrite,
            Permissions::ReadExecute,
            Permissions::ReadWriteExecute,
        ] {
            for owner in [Owner::User, Owner::Kernel] {
                for scope in [Scope::Global, Scope::Unique] {
                    let exp_mapping =
                        LeafMapping::new(PhysicalAddress(0x8020_0000), perms, owner, scope, 0x1000);
                    let entry = Sv39Entry::from(exp_mapping);
                    let act_mapping = entry.decode(Level::Leaf).unwrap();
                    match act_mapping {
                        Mapping::Leaf(m) => assert_eq!(exp_mapping, m),
                        _ => panic!("Not a leaf mapping"),
                    }
                }
            }
        }
    }

    #[test]
    fn map_and_translate() {
        let mut mgr = Sv39Manager::new(test_alloc);
        let vaddr = VirtualAddress(0x8020_0000);
        let mapping = LeafMapping::new(
            PhysicalAddress(0x1000),
            Permissions::ReadWrite,
            Owner::Kernel,
            Scope::Global,
            4096,
        );

        mgr.map(vaddr, mapping).expect("Failed to map");
        assert_eq!(mapping, mgr.translate(vaddr).unwrap());
    }
}
