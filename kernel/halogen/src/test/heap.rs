#[allow(unused_imports)]
use crate::{mem::heap, prelude::*};


#[test_case]
fn independent_allocs() {
    let v1: Vec<usize> = vec![1; 1000];
    let mut v2: Vec<usize> = vec![2; 1000];
    let v3: Vec<usize> = vec![3; 1000];

    assert!(ptr::addr_of!(*v1.last().unwrap()) < ptr::addr_of!(*v2.first().unwrap()));
    assert!(ptr::addr_of!(*v2.last().unwrap()) < ptr::addr_of!(*v3.first().unwrap()));

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

#[test_case]
fn kmalloc_stress() {
    let sizes = [16, 64, 512, KIB, 10 * KIB];

    let trials = 10;

    for _ in 0..trials {
        sizes
            .iter()
            .map(|&size| heap::kmalloc(size).expect("kmalloc failed"))
            .collect::<Vec<_>>() // Collect to evaluate all kmalloc()s before any of the kfree()s
            .iter()
            .for_each(|ptr| unsafe { heap::kfree(*ptr) });
    }
}
