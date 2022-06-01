use goblin::elf::Elf;
use halogen_common::mem::{Address, VirtualAddress};

use crate::{
    error::{KernelError, KernelResult},
    kerror,
    mem::{
        paging::{Level, Permissions, Privilege, Scope, PAGE_SIZE},
        phys, AddressSpace,
    },
};

pub fn load_elf(space: &mut AddressSpace, elf_bytes: &[u8]) -> KernelResult<()> {
    let elf = match Elf::parse(elf_bytes) {
        Ok(elf) => elf,
        Err(_) => return kerror!(KernelError::ExecutableFormat).into(),
    };

    // For each program section...
    for phdr in elf.program_headers.iter() {
        if phdr.vm_range().is_empty() {
            continue;
        }

        let section_start = phdr.file_range().start;

        // Get the permissions.
        let perms = match (phdr.is_read(), phdr.is_write(), phdr.is_executable()) {
            // R     W     X
            (true, true, false) => Permissions::ReadWrite,
            (true, false, false) => Permissions::ReadOnly,
            (true, false, true) | (false, false, true) => Permissions::ReadExecute,
            _ => return kerror!(KernelError::ExecutableFormat).into(),
        };

        // For each page in that section...
        for (offset, virt_addr) in phdr.vm_range().into_iter().enumerate().step_by(PAGE_SIZE) {
            // Allocate a page.
            let (virt_page, phys_page) =
                phys::alloc().ok_or_else(|| kerror!(KernelError::OutOfPhysicalFrames))?;

            // Get virtual pointers to the source and destination.
            let src_ptr: *const u8 = unsafe { elf_bytes.as_ptr().add(section_start + offset) };
            let dest_ptr: *mut u8 = virt_page.as_mut_ptr();

            // Copy from the source to the new page and map it into the address space.
            unsafe {
                core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, PAGE_SIZE);
                space.root.map(
                    VirtualAddress(virt_addr),
                    phys_page,
                    Level::Page,
                    perms,
                    Scope::Local,
                    Privilege::User,
                )?;
            }
        }
    }

    Ok(())
}
