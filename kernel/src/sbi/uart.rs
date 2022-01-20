use core::fmt::{Arguments, Error, Write};

use crate::sbi::{sbi_ecall, UART_EXT_ID, UART_PUTC_FUNC_ID};

/// Simple handle for the UART device
struct UartWriter;

// Implement the `Write` trait so we can print format strings
impl Write for UartWriter {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for b in s.bytes() {
            sbi_ecall([b as usize, 0, 0, 0, 0, 0], UART_EXT_ID, UART_PUTC_FUNC_ID)
                .expect("error when printing character");
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    UartWriter.write_fmt(args).unwrap();
}

/// Print a string over UART
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::sbi::uart::_print(format_args!($($arg)*)));
}

/// Print a string and newline over UART
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
