// Source: Phillip Opperman, https://os.phil-opp.com/testing/

use crate::{exit_failure, exit_success, print, println};
use core::panic::PanicInfo;

pub trait TestCase {
    fn run(&self) -> ();
}

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
        println!("ok");
    }
}

pub fn test_runner(tests: &[&dyn TestCase]) -> ! {
    println!();
    for test in tests {
        test.run();
    }
    exit_success!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("failed!\n");
    println!("Error: {}\n", info);
    exit_failure!();
}
