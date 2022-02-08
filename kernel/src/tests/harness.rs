use crate::{print, println};

pub trait TestCase {
    fn run(&self);
}

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("{}...", core::any::type_name::<T>());
        self();
        println!("ok");
    }
}

#[cfg(test)]
pub fn run_tests(tests: &[&dyn TestCase]) -> ! {
    println!("\nRunning {} tests", tests.len());

    for test in tests {
        test.run();
    }

    crate::exit!(0);
}
