#[cfg(not(test))]
mod _panic {

    use crate::println;
    use core::panic::PanicInfo;

    #[cfg(dump_on_panic)]
    use crate::{print_dump, text_begin, text_size};

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        println!("\n{:}\n", info);

        let ra: usize;
        let sp: usize;
        let gp: usize;
        let fp: usize;

        unsafe {
            asm!("mv {}, ra", out(reg) ra);
            asm!("mv {}, sp", out(reg) sp);
            asm!("mv {}, fp", out(reg) fp);
            asm!("mv {}, gp", out(reg) gp);
        }

        println!("ra = 0x{:08X}", ra);
        println!("sp = 0x{:08X}", sp);
        println!("fp = 0x{:08X}", fp);
        println!("gp = 0x{:08X}\n", gp);

        #[cfg(dump_on_panic)]
        print_dump(text_begin!(), text_size!());

        loop {}
    }
}
