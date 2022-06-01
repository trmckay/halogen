use crate::{arch::Context, log::*};

mod print;
mod task;

#[derive(Clone, Copy, Debug)]
pub enum Function {
    Exit,
    Print,
    Invalid,
}

impl From<usize> for Function {
    fn from(n: usize) -> Function {
        match n {
            0 => Function::Exit,
            1 => Function::Print,
            _ => Function::Invalid,
        }
    }
}

/// # Safety
///
/// Should only be called from the trap handler
pub unsafe extern "C" fn handle_syscall(ctx: &mut Context) {
    let syscall_fn = Function::from(ctx.gp_regs[9]);
    let a1 = ctx.gp_regs[10];
    let a2 = ctx.gp_regs[11];
    let a3 = ctx.gp_regs[12];
    let a4 = ctx.gp_regs[13];

    trace!(
        "Handle system call: {:?}(0x{:x}, 0x{:x}, 0x{:x}, 0x{:x})",
        syscall_fn,
        a1,
        a2,
        a3,
        a4
    );

    // Want to return to the next instruction, not the `ecall`.
    ctx.pc += 0x4;

    let ret = match syscall_fn {
        Function::Exit => task::syscall_exit(a1 as isize),
        Function::Print => print::syscall_print(a1 as *const u8, a2),
        Function::Invalid => -1,
    };

    ctx.gp_regs[9] = ret as usize;
}
