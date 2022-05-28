use super::call::sbi_ecall;

const CONSOLE_PUTCHAR_EXT_ID: usize = 0x01;
const CONSOLE_GETCHAR_EXT_ID: usize = 0x02;

/// Zero-size structure that implements `Write` using SBI calls.
pub struct SbiConsole;

impl core::fmt::Write for SbiConsole {
    fn write_str(&mut self, str: &str) -> core::fmt::Result {
        let mut args = [0; 6];
        for b in str.bytes() {
            args[0] = b as usize;
            if sbi_ecall(CONSOLE_PUTCHAR_EXT_ID, 0, args).is_err() {
                return Err(core::fmt::Error);
            };
        }
        Ok(())
    }
}
