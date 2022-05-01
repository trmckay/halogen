use crate::prelude::*;

pub trait TestCase {
    fn run(&self);
}

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self) {
        println!("---");
        println!("Running test {}", core::any::type_name::<T>());

        self();

        println!("Test complete");
    }
}

pub fn run_tests(tests: &[&dyn TestCase]) -> ! {
    println!("\nRunning {} tests\n", tests.len());
    log::set_level(log::Level::Trace);

    for test in tests {
        test.run();
    }

    println!("---\n\nAll tests passed");

    exit!(0);
}
