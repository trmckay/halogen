use owo_colors::{colors, OwoColorize};

use crate::{
    kprintln,
    sbi::reset::{shutdown, Reason},
};

pub trait TestCase {
    fn run(&self);
}

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self) {
        kprintln!("---");
        kprintln!("{}", core::any::type_name::<T>().fg::<colors::Cyan>());

        self();

        kprintln!("{}", "Ok".fg::<colors::Green>());
    }
}

pub fn run_tests(tests: &[&dyn TestCase]) -> ! {
    kprintln!("\nRunning {} tests\n", tests.len());

    for test in tests {
        test.run();
    }

    kprintln!("---\n\nAll tests passed");

    shutdown(Reason::None);
}
