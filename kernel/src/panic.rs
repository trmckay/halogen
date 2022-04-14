use core::panic::PanicInfo;

use crate::{prelude::*, println_unsafe};

/// Use the test device to shutdown/exit the machine
#[macro_export]
macro_rules! exit {
    ($c:expr) => {
        use qemu_exit::*;
        RISCV64::new(($crate::mem::DEV_TEST + $crate::mem::MMIO_OFFSET) as u64).exit($c);
    };
}

#[panic_handler]
unsafe fn panic(panic: &PanicInfo) -> ! {
    // Maybe we panicked while the UART is locked
    println_unsafe!(
        "\n{}{}{}",
        Style::default().color(Color::Red),
        panic,
        Style::default()
    );
    exit!(1);
}
