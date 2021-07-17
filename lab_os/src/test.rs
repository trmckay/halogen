use crate::driver;
use crate::{print, println};

#[cfg(test)]
pub fn test(tests: &[&dyn Fn()]) {
    let mut uart = driver::UartDriver::new(driver::DEV_UART);
    uart.init();

    for (i, t) in tests.iter().enumerate() {
        println!(uart, "Test {}: ", i);
    }

    println!(uart, "Test cases completed.");
}
