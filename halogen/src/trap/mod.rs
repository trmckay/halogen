global_asm!(include_str!("trap.s"));

/// CPU trap-handler. When the CPU issues a trap, it will jump
/// here. This should handle the cause and return the next program counter.
#[no_mangle]
pub extern "C" fn mtrap_vector() -> ! {
    loop {}
}
