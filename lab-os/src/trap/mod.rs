use crate::cpu::CpuFrame;
use crate::println;

global_asm!(include_str!("trap.s"));

/// CPU trap-handler. When the CPU issues a trap, it will jump
/// here. This should handle the cause and return the next program counter.
#[no_mangle]
pub extern "C" fn mtrap_vector(
    mepc: usize,
    mtval: usize,
    cause: usize,
    hart: usize,
    status: usize,
    frame: &mut CpuFrame,
) -> usize {
    println!("CPU trap:");
    println!("mepc:    {:p}", mepc as *const u8);
    println!("mtval:   {:p}", mtval as *const u8);
    println!("cause:   {:b}", cause);
    println!("hart:    {}", hart);
    println!("status:  {}", status);

    println!("\n{:?}", frame);

    loop {}
}
