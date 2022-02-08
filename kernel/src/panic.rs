use core::panic::PanicInfo;

use crate::println;

#[macro_export]
macro_rules! exit {
    ($c:expr) => {
        use qemu_exit::*;
        RISCV64::new(crate::mem::MMIO_DEV_TEST as u64).exit($c);
    };
}

#[panic_handler]
fn panic(panic: &PanicInfo) -> ! {
    println!("\n{}", panic);
    exit!(1);
}
