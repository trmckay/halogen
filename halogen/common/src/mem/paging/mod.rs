//! This module and the traits within act as a shim between the rest of the
//! kernel and the specifics of Sv39 (or any other paging scheme). This has the
//! benefits of both abstracting away the scheme and allowing for more
//! experimentation with the underlying implementation.

use super::{PhysicalAddress, VirtualAddress};

/// Implementation of the Sv39 spec.
pub mod sv39;

/// Valid permissions for mappings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permissions {
    ReadOnly,
    ReadExecute,
    ReadWrite,
    ReadWriteExecute,
    Nothing,
    Invalid,
}

/// Describes in which address spaces a mapping is accessible. Global mappings
/// should be mapped in all page tables (used for the kernel half).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Global,
    Unique,
}

/// Differentiate between user and kernel mappings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    Kernel,
    User,
}

/// A leaf mapping that can be translated to a physical address. Different
/// platforms may encode this information differently, but should all contain
/// this information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeafMapping {
    addr: PhysicalAddress,
    perms: Permissions,
    owner: Owner,
    scope: Scope,
    size: usize,
}

impl LeafMapping {
    pub fn new(
        addr: PhysicalAddress,
        perms: Permissions,
        owner: Owner,
        scope: Scope,
        size: usize,
    ) -> LeafMapping {
        LeafMapping {
            addr,
            perms,
            owner,
            scope,
            size,
        }
    }
}

/// Non-leaf mapping that translates to a next-level page table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DirectoryMapping {
    next: PhysicalAddress,
    scope: Scope,
}

impl DirectoryMapping {
    pub fn new(next: PhysicalAddress, scope: Scope) -> DirectoryMapping {
        DirectoryMapping { next, scope }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mapping {
    Leaf(LeafMapping),
    Directory(DirectoryMapping),
}

/// A page table that stores virtual address mappings. `map` and `translate`
/// should be inverses of each other, i.e.
///
/// ```
/// pt.translate(pt.map(vaddr, mapping)) == mapping
/// ```
///
/// TODO: Is another function needed for altering mappings.
pub trait PagingScheme {
    /// Map a virtual address to the address/metadata in `mapping`. Overwrites
    /// any existing mappings.
    fn map(&mut self, vaddr: VirtualAddress, mapping: LeafMapping) -> Result<(), PagingError>;

    /// Translate a virtual address and return the leaf mapping if it exists.
    fn translate(&self, vaddr: VirtualAddress) -> Option<LeafMapping>;

    /// Unmap a virtual address.
    ///
    /// TODO: This API is may not work.
    fn unmap(&mut self, vaddr: VirtualAddress) -> Result<(), PagingError>;
}

#[derive(Clone, Copy, Debug)]
pub enum PagingError {
    /// Misaligned virtual address.
    MisalignedVirtAddr,
    /// Misaligned phyiscal address
    MisalignedPhysAddr,
    /// Failed to allocate a next-level page-table while mapping.
    AllocationError,
    /// The page-table is corrupted in some way.
    PageTableCorruption,
    /// Mapping request is invalid or is unsupported.
    InvalidRequest,
}
