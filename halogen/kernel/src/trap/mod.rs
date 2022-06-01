use halogen_common::mem::{VirtualAddress, KIB};

use crate::{
    arch::Context,
    fwprintln,
    io::console::{early_print, early_println},
    irq::plic,
    log::*,
    mem::{regions::Region, Stack},
    read_csr,
    sbi::reset::{shutdown, Reason},
    syscall::handle_syscall,
    task::{executor::timer_event, resume},
};

/// Set the trap vector and allocate a stack for context saving
///
/// # Safety
///
/// - Call before enabling interrupts
/// - Only call once
pub unsafe fn init() {
    info!("Initialize trap handler");
    riscv::register::stvec::write(trap_shim as usize, riscv::register::stvec::TrapMode::Direct);

    let stack = Stack::try_new_kernel(24 * KIB).expect("failed to allocate trap stack");
    riscv::register::sscratch::write(stack.top() as usize);
}

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

/// Trap for exceptions early in the boot process
///
/// # Safety
///
/// This should never be called directly
#[repr(align(4))]
pub unsafe extern "C" fn early_trap() -> ! {
    early_print("\nAn exception occurred during bootstrap: ");

    let scause = read_csr!(scause);

    // We don't use a `match` here because the generated code is not PIC
    if scause == 1 {
        early_println("Fetch fault");
    } else if scause == 2 {
        early_println("Illegal instruction");
    } else if scause == 5 {
        early_println("Load fault");
    } else if scause == 7 {
        early_println("Store fault");
    } else if scause == 12 {
        early_println("Fetch page-fault");
    } else if scause == 13 {
        early_println("Load page-fault");
    } else if scause == 14 {
        early_println("Store page-fault");
    } else {
        early_println("Other exception");
    }

    shutdown(Reason::Failure);
}

/// Save the context and call trap handler.
#[repr(align(4))]
#[naked]
unsafe extern "C" fn trap_shim() -> ! {
    core::arch::asm!(include_str!("ctx_swap.s"), options(noreturn));
}

fn dump_ctx(ctx: &Context, scause: TrapCause, stval: usize) {
    fwprintln!("scause = {:?}", scause);
    fwprintln!(
        "stval = {}, {:p} ({:?})",
        stval,
        stval as *const u8,
        Region::from(VirtualAddress(stval))
    );
    fwprintln!("{}", ctx);
}

/// Handle the trap/interrupt/exception. Returns a `Context` which contains the
/// general purpose registers, calling environment, and program counter.
#[no_mangle]
unsafe extern "C" fn trap_handler(
    ctx: *mut Context,
    scause: usize,
    stval: usize,
) -> *const Context {
    let ctx = ctx.as_mut().unwrap();
    let scause: TrapCause = scause.into();

    match scause {
        TrapCause::SupervisorExternal => {
            plic::handle_next();
        }
        TrapCause::SupervisorTimer => {
            timer_event();
        }
        TrapCause::UserCall => handle_syscall(ctx),
        _ => {
            // TODO: Don't just panic; kill the current thread if it isn't TID=0
            dump_ctx(ctx, scause, stval);
            shutdown(Reason::Failure);
        }
    }

    resume(ctx)
}
