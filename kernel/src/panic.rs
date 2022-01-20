use core::panic::PanicInfo;

use crate::println;

#[panic_handler]
fn panic(panic: &PanicInfo) -> ! {
    println!("{}", panic);
    loop {}
}
