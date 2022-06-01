use crate::task;

#[test_case]
fn get_tid() {
    assert_eq!(0, task::tid());
}

/// Exponentially multithreaded Fibonacci implementation
extern "C" fn fib(n: usize) -> isize {
    match n {
        0 | 1 => 1,
        n => {
            let t1 = task::spawn(fib, n - 2).unwrap();
            let t2 = task::spawn(fib, n - 1).unwrap();

            let op1 = task::join(t1).unwrap();
            let op2 = task::join(t2).unwrap();

            op1 + op2
        }
    }
}

/// A fast iterator-based approach to check against
struct FibIterator {
    curr: isize,
    next: isize,
}

impl Iterator for FibIterator {
    type Item = isize;

    fn next(&mut self) -> Option<isize> {
        let next = self.curr + self.next;
        let prev = self.curr;
        self.curr = self.next;
        self.next = next;
        Some(prev)
    }
}

impl Default for FibIterator {
    fn default() -> FibIterator {
        FibIterator { curr: 1, next: 1 }
    }
}

/// Test driver
fn fib_test(n: usize) {
    FibIterator::default()
        .take(n)
        .enumerate()
        .for_each(|(i, v)| assert_eq!(v, fib(i)))
}

#[test_case]
fn fib_multithread_1() {
    fib_test(1)
}

#[test_case]
fn fib_multithread_2() {
    fib_test(2)
}

#[test_case]
fn fib_multithread_4() {
    fib_test(4)
}

#[test_case]
fn fib_multithread_8() {
    fib_test(8)
}
