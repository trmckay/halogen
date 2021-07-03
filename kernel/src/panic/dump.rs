use crate::{print, println};
use core::ptr::read;

// Prints a human-readable dump of memory.
pub fn print_dump(base: usize, size: usize) {
    let mut buf: [u8; 16] = [0; 16];

    for offset in 0..size {
        let addr = base + offset;

        if (offset % 16 == 0 || offset == size - 1) && offset != 0 {
            print!("{:08x}..{:08x}: ", addr, addr + 15);
            for (i, c) in buf.iter().enumerate() {
                if i % 4 == 0 {
                    print!(" ");
                }
                print!("{:02x}", *c);
            }

            print!("  ");

            for c in buf.iter() {
                print!(
                    "{}",
                    match *c {
                        0x20..0x7e => *c as char,
                        _ => '.',
                    }
                );
            }

            println!();
        }

        unsafe {
            buf[(offset % 16) as usize] = read(addr as *const u8);
        }
    }
}
