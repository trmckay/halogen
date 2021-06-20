use crate::{print, println, util::print_dump};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let pc: usize;
    let sp: usize;
    let gp: usize;
    let fp: usize;

    // Save some special registers.
    unsafe {
        asm!("mv {}, ra", out(reg) pc);
        asm!("mv {}, sp", out(reg) sp);
        asm!("mv {}, fp", out(reg) fp);
        asm!("mv {}, gp", out(reg) gp);
    }

    // Print debug info.
    println!("Kernel panic!\n");

    println!("ra = 0x{:08X}", pc);
    println!("sp = 0x{:08X}", sp);
    println!("fp = 0x{:08X}", fp);
    println!("gp = 0x{:08X}\n", gp);

    // Dump the stack.
    let stack_end: usize;
    unsafe {
        asm!("la {}, _stack_end", out(reg) stack_end);
    }
    println!("Stack dump:\n");
    print_dump(sp, stack_end - sp);

    // Restart
    loop {}
}
