use crate::thread;

#[test_case]
fn get_tid() {
    assert_eq!(0, thread::tid());
}

extern "C" fn fib(n: usize) -> usize {
    match n {
        0 | 1 => 1,
        n => {
            let t1 = thread::spawn(fib, n - 2).unwrap();
            let t2 = thread::spawn(fib, n - 1).unwrap();

            let op1 = thread::join(t1).unwrap();
            let op2 = thread::join(t2).unwrap();

            op1 + op2
        }
    }
}

#[test_case]
fn spawn_and_join() {
    let n = 6;
    let tid = thread::spawn(fib, n).expect("failed to spawn thread");
    assert_eq!(13, thread::join(tid).expect("failed to join thread"));
}
