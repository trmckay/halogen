use core::panic::PanicInfo;

use crate::prelude::*;

/// Use the test device to shutdown/exit the machine
#[macro_export]
macro_rules! exit {
    ($c:expr) => {
        use qemu_exit::*;
        info!("Exiting with code {}", $c);
        RISCV64::new(($crate::mem::DEV_TEST + $crate::mem::MMIO_OFFSET) as u64).exit($c);
    };
}

#[panic_handler]
unsafe fn panic(panic: &PanicInfo) -> ! {
    error!("Hart {} {}", hart_id!(), panic);
    exit!(1);
}
