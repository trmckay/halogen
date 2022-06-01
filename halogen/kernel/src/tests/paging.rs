use crate::mem::{
    paging::{translate, Permissions, Privilege, Scope},
    regions::{KERNEL_SPACE_START, PHYSICAL_BASE},
};

#[test_case]
fn address_translation() {
    let (phys_addr, _, _, _) = translate(KERNEL_SPACE_START).unwrap();
    assert_eq!(unsafe { PHYSICAL_BASE }, phys_addr);
}

#[test_case]
fn scope_translation() {
    let (_, scope, _, _) = translate(KERNEL_SPACE_START).unwrap();
    assert_eq!(Scope::Global, scope);
}

#[test_case]
fn privilege_translation() {
    let (_, _, prv, _) = translate(KERNEL_SPACE_START).unwrap();
    assert_eq!(Privilege::Kernel, prv);
}

#[test_case]
fn permissions_translation() {
    let (_, _, _, perms) = translate(KERNEL_SPACE_START).unwrap();
    assert_eq!(Permissions::ReadExecute, perms);
}
