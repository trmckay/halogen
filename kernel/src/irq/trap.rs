use crate::{arch::Context, irq::plic, prelude::*, thread};

#[derive(Debug, Clone, Copy)]
enum TrapCause {
    FetchMisaligned,
    FetchFault,
    IllegalInstruction,
    Breakpoint,
    LoadMisaligned,
    LoadFault,
    StoreMisaligned,
    StoreFault,
    UserCall,
    SupervisorCall,
    FetchPageFault,
    LoadPageFault,
    StorePageFault,
    UserSoftware,
    SupervisorSoftware,
    MachineSoftware,
    UserTimer,
    SupervisorTimer,
    MachineTimer,
    UserExternal,
    SupervisorExternal,
    MachineExternal,
}

impl From<usize> for TrapCause {
    /// Create a `TrapCause` enum from the interrupt cause code
    fn from(cause: usize) -> TrapCause {
        let is_async = cause >> 63 != 0;
        let num = cause & 0xFFF;

        match (is_async, num) {
            (false, 0) => TrapCause::FetchMisaligned,
            (false, 1) => TrapCause::FetchFault,
            (false, 2) => TrapCause::IllegalInstruction,
            (false, 3) => TrapCause::Breakpoint,
            (false, 4) => TrapCause::LoadMisaligned,
            (false, 5) => TrapCause::LoadFault,
            (false, 6) => TrapCause::StoreMisaligned,
            (false, 7) => TrapCause::StoreFault,
            (false, 8) => TrapCause::UserCall,
            (false, 9) => TrapCause::SupervisorCall,
            (false, 12) => TrapCause::FetchPageFault,
            (false, 13) => TrapCause::LoadPageFault,
            (false, 15) => TrapCause::StorePageFault,
            (true, 0) => TrapCause::UserSoftware,
            (true, 1) => TrapCause::SupervisorSoftware,
            (true, 3) => TrapCause::MachineSoftware,
            (true, 4) => TrapCause::UserTimer,
            (true, 5) => TrapCause::SupervisorTimer,
            (true, 7) => TrapCause::MachineTimer,
            (true, 8) => TrapCause::UserExternal,
            (true, 9) => TrapCause::SupervisorExternal,
            (true, 11) => TrapCause::MachineExternal,
            _ => panic!("invalid trap : async={} num={}", is_async, num),
        }
    }
}

/// Trap vector from supervisor mode
///
/// # Safety
///
/// This should never be called directly
#[repr(align(4))]
#[naked]
pub unsafe extern "C" fn trap_shim() -> ! {
    asm!(
        "
        csrci sstatus, 1

        # Swap sp with scratch stack
        # sscratch <- sp
        csrrw sp, sscratch, sp

        # ctx: mut Context = (uninitialized);
        # sp: *mut Context <- &ctx;
        addi sp, sp, (33 * -8)

        # ctx.registers <- GP registers
        sd x1, (0 * 8)(sp)
        # Skip x2/sp
        sd x3, (2 * 8)(sp)
        sd x4, (3 * 8)(sp)
        sd x5, (4 * 8)(sp)
        sd x6, (5 * 8)(sp)
        sd x7, (6 * 8)(sp)
        sd x8, (7 * 8)(sp)
        sd x9, (8 * 8)(sp)
        sd x10, (9 * 8)(sp)
        sd x11, (10 * 8)(sp)
        sd x12, (11 * 8)(sp)
        sd x13, (12 * 8)(sp)
        sd x14, (13 * 8)(sp)
        sd x15, (14 * 8)(sp)
        sd x16, (15 * 8)(sp)
        sd x17, (18 * 8)(sp)
        sd x18, (17 * 8)(sp)
        sd x19, (18 * 8)(sp)
        sd x20, (19 * 8)(sp)
        sd x21, (20 * 8)(sp)
        sd x22, (21 * 8)(sp)
        sd x23, (22 * 8)(sp)
        sd x24, (23 * 8)(sp)
        sd x25, (24 * 8)(sp)
        sd x26, (25 * 8)(sp)
        sd x27, (26 * 8)(sp)
        sd x28, (27 * 8)(sp)
        sd x29, (28 * 8)(sp)
        sd x30, (29 * 8)(sp)
        sd x31, (30 * 8)(sp)

        # ctx.registers.sp <- sscratch
        csrr t0, sscratch
        sd t0, (1 * 8)(sp)

        # ctx.pc <- sepc
        csrr t0, sepc
        sd t0, (31 * 8)(sp)

        # TODO: check CSRs for calling environment
        # ctx.env <- Environment::Supervisor
        li t0, 1
        sd t0, (32 * 8)(sp)

        # Disable interrupts
        li t0, 0b10000
        csrs sstatus, t0

        # Prepare arguments for trap_handler()
        mv a0, sp
        csrr a1, scause
        csrr a2, stval
        csrr a3, sepc

        # ctx: *const Context = trap_handler(ctx, scause, stval, sepc)
        # a0 <- ctx
        call trap_handler

        # sepc <- ctx.pc
        ld t0, (31 * 8)(a0)
        csrrw t0, sepc, t0

        # GP registers <- ctx.registers
        ld x1, (0 * 8)(a0)
        ld x2, (1 * 8)(a0)
        ld x3, (2 * 8)(a0)
        ld x4, (3 * 8)(a0)
        ld x5, (4 * 8)(a0)
        ld x6, (5 * 8)(a0)
        ld x7, (6 * 8)(a0)
        ld x8, (7 * 8)(a0)
        ld x9, (8 * 8)(a0)
        # Save x10/a0 for last
        ld x11, (10 * 8)(a0)
        ld x12, (11 * 8)(a0)
        ld x13, (12 * 8)(a0)
        ld x14, (13 * 8)(a0)
        ld x15, (14 * 8)(a0)
        ld x16, (15 * 8)(a0)
        ld x17, (18 * 8)(a0)
        ld x18, (17 * 8)(a0)
        ld x19, (18 * 8)(a0)
        ld x20, (19 * 8)(a0)
        ld x21, (20 * 8)(a0)
        ld x22, (21 * 8)(a0)
        ld x23, (22 * 8)(a0)
        ld x24, (23 * 8)(a0)
        ld x25, (24 * 8)(a0)
        ld x26, (25 * 8)(a0)
        ld x27, (26 * 8)(a0)
        ld x28, (27 * 8)(a0)
        ld x29, (28 * 8)(a0)
        ld x30, (29 * 8)(a0)
        ld x31, (30 * 8)(a0)
        ld a0, (9 * 8)(a0)

        # Return from the trap/interrupt/exception
        sret
    ",
        options(noreturn)
    );
}

/// Handle the trap/interrupt/exception
///
/// Returns a `Context` which contains the general purpose registers, calling
/// environment, and program counter
#[no_mangle]
unsafe extern "C" fn trap_handler(
    ctx: *mut Context,
    scause: usize,
    stval: usize,
    sepc: usize,
) -> *const Context {
    let scause: TrapCause = scause.into();

    match scause {
        TrapCause::SupervisorExternal => plic::handle_pending(),
        TrapCause::SupervisorTimer => thread::timer_event(),
        _ => {
            // TODO: Don't just panic; kill the current thread if it isn't TID=0
            panic!(
                "unandled exception: cause={:?}, sepc=0x{:x}, stval=0x{:x}, ctx={:?}",
                scause, sepc, stval, *ctx
            )
        }
    }

    thread::resume(ctx)
}
