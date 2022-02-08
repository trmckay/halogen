use core::arch::asm;

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
enum InterruptType {
    Software = 1,
    Timer = 5,
    External = 9,
}

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
enum TrapCause {
    FetchMisaligned = 0,
    FetchFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadMisaligned = 4,
    LoadFault = 5,
    StoreMisaligned = 6,
    StoreFault = 7,
    UserCall = 8,
    SupervisorCall = 9,
    FetchPageFault = 12,
    LoadPageFault = 13,
    StorePageFault = 15,
    Unknown,
}

impl From<usize> for TrapCause {
    fn from(cause: usize) -> TrapCause {
        match cause {
            0 => TrapCause::FetchMisaligned,
            1 => TrapCause::FetchFault,
            2 => TrapCause::IllegalInstruction,
            3 => TrapCause::Breakpoint,
            4 => TrapCause::LoadMisaligned,
            5 => TrapCause::LoadFault,
            6 => TrapCause::StoreMisaligned,
            7 => TrapCause::StoreFault,
            8 => TrapCause::UserCall,
            9 => TrapCause::SupervisorCall,
            12 => TrapCause::FetchPageFault,
            13 => TrapCause::LoadPageFault,
            15 => TrapCause::StorePageFault,
            _ => TrapCause::Unknown,
        }
    }
}

#[repr(align(4))]
pub unsafe extern "C" fn trap_handler() {
    let scause: usize;
    let stval: usize;
    let sepc: usize;

    asm!("csrr {}, scause", out(reg) scause);
    asm!("csrr {}, stval", out(reg) stval);
    asm!("csrr {}, sepc", out(reg) sepc);

    let scause: TrapCause = scause.into();

    panic!(
        "Supervisor trap: {:?}, stval = 0x{:x}, sepc = 0x{:x}",
        scause, stval, sepc
    );
}
