use core::panic::PanicInfo;

use owo_colors::{AnsiColors, OwoColorize, Style};

use crate::{
    fwprint, fwprintln,
    io::console::early_println,
    mem::paging::PAGING_ENABLED,
    sbi::reset::{shutdown, Reason},
};

#[panic_handler]
unsafe fn panic(panic: &PanicInfo) -> ! {
    if PAGING_ENABLED {
        let red = Style::new().color(AnsiColors::Red);
        fwprint!("{}", "\nKernel panic: ".style(red.bold()));
        match panic.message() {
            Some(args) => fwprint!("{} ", args.style(red)),
            None => fwprint!("{} ", "no message".style(red)),
        }
        if let Some(location) = panic.location() {
            fwprintln!("{}", format_args!("-- {}", location).style(red));
        }
    } else {
        early_println("\nKernel panicked during bootstrap");
    }

    shutdown(Reason::Failure);
}
