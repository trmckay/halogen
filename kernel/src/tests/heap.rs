use core::{
    mem::size_of,
    ptr::{addr_of, addr_of_mut, null_mut},
};

use crate::mem::heap::*;

#[test_case]
fn test_layout() {
    let alloc_a: Box<u128> = Box::new(0);
    let alloc_b: Box<u128> = Box::new(0);
    let alloc_c: Box<u128> = Box::new(0);

    core::mem::drop(alloc_c);
    core::mem::drop(alloc_a);
    core::mem::drop(alloc_b);

    assert!(heap_empty())
}

#[test_case]
fn test_free() {
    {
        let addr_a;
        let a = Box::new(1);
        addr_a = addr_of!(*a);

        core::mem::drop(a);

        let b = Box::new(1);
        let addr_b = addr_of!(*b);

        assert_eq!(addr_a, addr_b);
    }
    assert!(heap_empty())
}
