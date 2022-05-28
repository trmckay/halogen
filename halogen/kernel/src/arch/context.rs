use halogen_common::mem::VirtualAddress;

use super::REGISTER_NAMES;
use crate::{
    mem::regions::Region,
    read_reg,
    task::thread::{ThreadFunction, ThreadShim},
};

/// Privilege level that the hart was running at.
#[repr(usize)]
#[derive(Clone, Copy, Debug)]
pub enum Privilege {
    Machine = 0,
    Supervisor = 1,
    User = 2,
}

/// Stores a CPU state for interrupts/traps, context switches, etc.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Context {
    pub registers: [usize; 31],
    pub pc: usize,
    pub prv: Privilege,
}

impl core::fmt::Display for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let registers = self.registers;
        let prv = self.prv;

        writeln!(f, "Mode = {:?}", prv)?;
        writeln!(
            f,
            "pc = {:p} ({:?})",
            self.pc as *const u8,
            Region::from(VirtualAddress(self.pc))
        )?;

        for (i, &r) in registers.iter().enumerate() {
            writeln!(
                f,
                "{} = {}, {:p} ({:?})",
                REGISTER_NAMES[i + 1],
                r,
                r as *const u8,
                Region::from(VirtualAddress(r))
            )?;
        }
        Ok(())
    }
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Default for Context {
    /// Create a new context with registers `gp` and `tp` configured for use in
    /// kernel-space on the calling hart.
    fn default() -> Context {
        let mut ctx = Context {
            registers: [0; 31],
            pc: 0,
            prv: Privilege::Supervisor,
        };

        ctx.registers[2] = read_reg!(gp);
        ctx.registers[3] = read_reg!(tp);

        ctx
    }
}

impl Context {
    /// Load the initial state of a context such that the `shim` function will
    /// call `entry` with `arg` as the first argument and `sp` as the stack.
    pub fn prepare(&mut self, sp: *mut u8, shim: ThreadShim, entry: ThreadFunction, arg: usize) {
        self.pc = shim as usize;
        self.registers[9] = entry as usize;
        self.registers[10] = arg;
        self.registers[1] = sp as usize;
    }
}
