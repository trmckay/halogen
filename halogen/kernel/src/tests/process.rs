use halogen_common::mem::{Address, VirtualAddress};
use halogen_programs::HELLO;

use crate::{
    mem::{
        paging::{map, translate, PageTable, Permissions, Privilege, Scope, PAGE_SIZE},
        regions::IMAGE_TEXT,
    },
    task::{executor::exec, join, process::Process},
};

#[test_case]
fn page_table_clone_diff() {
    let pt = PageTable::from_kernel_root();
    let root_pte_ptr = (riscv::register::satp::read().ppn() << 12) as *const PageTable;
    assert_ne!(root_pte_ptr, core::ptr::addr_of!(pt));
}

#[test_case]
fn page_table_clone_aligned() {
    let pt = PageTable::from_kernel_root();
    assert!(core::ptr::addr_of!(pt) as usize % PAGE_SIZE == 0);
}

#[test_case]
fn page_table_clone_translates() {
    let pt = PageTable::from_kernel_root();

    assert_eq!(
        // Implicity uses the root page table.
        translate(IMAGE_TEXT.start).unwrap(),
        pt.translate(IMAGE_TEXT.start).unwrap()
    )
}

#[test_case]
fn load_elf() {
    let load_addr = VirtualAddress(0x1000);
    let first_word = u32::from_be(0x17f1ff7f);

    let proc = Process::try_from_elf(1, HELLO).unwrap();

    let (prog_phys, scope, prv, perms) = proc.space.root.translate(load_addr).unwrap();

    assert_eq!(scope, Scope::Local);
    assert_eq!(prv, Privilege::User);
    assert_eq!(perms, Permissions::ReadExecute);

    let prog_virt = unsafe {
        map(
            None,
            Some(prog_phys),
            PAGE_SIZE,
            Permissions::ReadOnly,
            Scope::Global,
            Privilege::Kernel,
        )
        .unwrap()
    };

    unsafe { assert_eq!(first_word, *prog_virt.as_ptr::<u32>()) }
}

#[test_case]
fn create_main() {
    let mut proc = Process::try_from_elf(1, HELLO).unwrap();
    let _ = proc.create_main(172).unwrap();
}

#[test_case]
fn exec() {
    let (_, tid) = exec(HELLO).expect("failed to exec");
    let status = join(tid).expect("failed to join on user thread");
    assert_eq!(-1, status);
}
