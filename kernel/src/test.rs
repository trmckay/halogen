// Source: Phillip Opperman, https://os.phil-opp.com/testing/

use crate::{ansi, exit_failure, exit_success, print, println};
use core::panic::PanicInfo;

pub trait TestCase {
    fn run(&self);
}

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("test {} ... ", core::any::type_name::<T>());
        self();
        println!("{}ok{}", ansi::Color::Green, ansi::Color::Reset);
    }
}

pub fn test_runner(tests: &[&dyn TestCase]) -> ! {
    println!("\nrunning {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_success!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}FAILED{}", ansi::Color::Red, ansi::Color::Reset);
    println!("\nError: {}\n", info);
    exit_failure!();
}
