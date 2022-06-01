use alloc::{vec, vec::Vec};

use halogen_common::mem::KIB;

use crate::mem::heap;

#[test_case]
fn independence() {
    let v1: Vec<usize> = vec![1; 1000];
    let mut v2: Vec<usize> = vec![2; 1000];
    let v3: Vec<usize> = vec![3; 1000];

    assert!(core::ptr::addr_of!(*v1.last().unwrap()) < core::ptr::addr_of!(*v2.first().unwrap()));
    assert!(core::ptr::addr_of!(*v2.last().unwrap()) < core::ptr::addr_of!(*v3.first().unwrap()));

    v1.iter().for_each(|e| assert_eq!(1, *e));
    v2.iter().for_each(|e| assert_eq!(2, *e));
    v3.iter().for_each(|e| assert_eq!(3, *e));

    v2.iter_mut().for_each(|e| *e = 4);

    v1.iter().for_each(|e| assert_eq!(1, *e));
    v2.iter().for_each(|e| assert_eq!(4, *e));
    v3.iter().for_each(|e| assert_eq!(3, *e));

    drop(v2);

    let mut v2: Vec<usize> = vec![1; 1004];

    v2.iter_mut().for_each(|e| *e = 4);

    v1.iter().for_each(|e| assert_eq!(1, *e));
    v2.iter().for_each(|e| assert_eq!(4, *e));
    v3.iter().for_each(|e| assert_eq!(3, *e));
}

fn alloc_stress() {
    let sizes = [16, 64, 512, KIB, 10 * KIB];

    let trials = 10;

    for _ in 0..trials {
        let _ = sizes
            .iter()
            .map(|&size| {
                let v: Vec<u8> = Vec::with_capacity(size);
                v
            })
            .collect::<Vec<_>>();
    }
}

#[test_case]
fn stress() {
    for _ in 0..100 {
        let stats = heap::stats().unwrap();
        let usage_before = stats.bytes_used;
        let free_before = stats.bytes_free;

        alloc_stress();

        let stats = heap::stats().unwrap();
        assert_eq!(usage_before, stats.bytes_used);
        assert_eq!(free_before, stats.bytes_free);
    }
}
