use crate::{debug::print_dump, print, println};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let ra: usize;
    let sp: usize;
    let gp: usize;
    let fp: usize;

    // Save some special registers.
    unsafe {
        asm!("mv {}, ra", out(reg) ra);
        asm!("mv {}, sp", out(reg) sp);
        asm!("mv {}, fp", out(reg) fp);
        asm!("mv {}, gp", out(reg) gp);
    }

    println!("Kernel panic!\n");

    println!("ra = 0x{:08X}", ra);
    println!("sp = 0x{:08X}", sp);
    println!("fp = 0x{:08X}", fp);
    println!("gp = 0x{:08X}\n", gp);

    let text: usize;
    let text_end: usize;
    unsafe {
        asm!("la {}, _MEM", out(reg) text);
        asm!("la {}, _K_STACK", out(reg) text_end);
    }
    println!("Text and data dump:\n");
    print_dump(text, text_end - text);

    // Restart
    loop {}
}
