use crate::{print, println};
use core::ptr::read;

/// Prints a human-readable dump of memory from
/// `base` to `base + size`.
pub fn print_dump(base: *const u8, size: usize) {
    unsafe {
        let mut buf: [u8; 16] = [0; 16];

        for offset in 0..size {
            let addr = base.add(offset);

            if (offset % 16 == 0 || offset == size - 1) && offset != 0 {
                print!("{:p}..{:p}: ", addr, addr.add(15));
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

            buf[(offset % 16) as usize] = read(addr as *const u8);
        }
    }
}
