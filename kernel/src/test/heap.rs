// NOTE: Possible cargo-clippy bug is marking these imports as unused
#[allow(unused_imports)]
use crate::{mem::heap, prelude::*};

#[test_case]
fn coalesce_blocks() {
    let space_before = heap::free_space();

    let p1 = heap::kmalloc(1000).unwrap();
    let p2 = heap::kmalloc(1000).unwrap();
    let p3 = heap::kmalloc(1000).unwrap();

    assert_eq!(3, heap::used_blocks());

    unsafe {
        heap::kfree(p2);
    }

    assert_eq!(2, heap::used_blocks());
    assert_eq!(2, heap::free_blocks());

    unsafe {
        heap::kfree(p3);
    }

    assert_eq!(1, heap::used_blocks());
    assert_eq!(1, heap::free_blocks());

    unsafe {
        heap::kfree(p1);
    }

    assert_eq!(0, heap::used_blocks());
    assert_eq!(1, heap::free_blocks());

    let p4 = heap::kmalloc(1000).unwrap();

    assert_eq!(p4, p1);

    unsafe {
        heap::kfree(p4);
    }

    assert_eq!(0, heap::used_blocks());
    assert_eq!(1, heap::free_blocks());

    let space_after = heap::free_space();

    assert_eq!(space_before, space_after);
}

#[test_case]
fn independent_allocs() {
    let v1: Vec<usize> = vec![1; 1000];
    let mut v2: Vec<usize> = vec![2; 1000];
    let v3: Vec<usize> = vec![3; 1000];

    v2.iter_mut().for_each(|e| *e = 0);

    v1.iter().for_each(|e| assert_eq!(1, *e));
    v2.iter().for_each(|e| assert_eq!(0, *e));
    v3.iter().for_each(|e| assert_eq!(3, *e));

    core::mem::drop(v2);

    let mut v2: Vec<usize> = vec![1; 1004];

    v2.iter_mut().for_each(|e| *e = 0);

    v1.iter().for_each(|e| assert_eq!(1, *e));
    v2.iter().for_each(|e| assert_eq!(0, *e));
    v3.iter().for_each(|e| assert_eq!(3, *e));
}

#[test_case]
fn kmalloc_stress() {
    let sizes = [16, 64, 512, KIB, 10 * KIB];

    let trials = 10;

    let space_init = heap::free_space();

    for _ in 0..trials {
        assert_eq!(1, heap::free_blocks());
        let space_pre_alloc = heap::free_space();

        sizes
            .iter()
            .map(|&size| heap::kmalloc(size).expect("kmalloc failed"))
            .collect::<Vec<_>>() // Collect to evaluate all kmalloc()s before any of the kfree()s
            .iter()
            .for_each(|ptr| unsafe { heap::kfree(*ptr) });

        assert_eq!(1, heap::free_blocks());
        assert_eq!(space_pre_alloc, heap::free_space());
    }

    assert_eq!(space_init, heap::free_space());
}
