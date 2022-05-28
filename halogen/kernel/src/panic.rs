use core::panic::PanicInfo;

use crate::{
    fwprintln,
    io::console::early_println,
    mem::paging::PAGING_ENABLED,
    sbi::reset::{shutdown, Reason},
};

#[panic_handler]
unsafe fn panic(panic: &PanicInfo) -> ! {
    if PAGING_ENABLED {
        fwprintln!("Kernel {}", panic);
    } else {
        early_println("\nKernel panicked during bootstrap");
    }

    shutdown(Reason::Failure);
}
