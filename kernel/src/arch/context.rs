use crate::{
    prelude::*,
    thread::{ThreadFunction, ThreadShim},
};

#[repr(usize)] // `usize` to simplify alignment
#[derive(Clone, Copy, Debug)]
pub enum Environment {
    Machine = 0,
    Supervisor = 1,
    User = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GpRegisters {
    ra: usize,
    sp: usize,
    gp: usize,
    tp: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    s0_fp: usize,
    s1: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,
}

/// Stores a CPU state for interrupts/traps, context switches, etc.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub registers: GpRegisters,
    pub pc: usize,
    pub env: Environment,
}

impl Default for Context {
    /// Create a new context from kernel code
    fn default() -> Context {
        let mut ctx = Context {
            registers: GpRegisters::default(),
            pc: 0,
            env: Environment::Supervisor,
        };

        ctx.registers.gp = read_reg!(gp);
        ctx.registers.tp = read_reg!(tp);

        ctx
    }
}

impl Context {
    pub fn prepare(&mut self, sp: *mut u8, shim: ThreadShim, entry: ThreadFunction, arg: usize) {
        self.pc = shim as usize;
        self.registers.a0 = entry as usize;
        self.registers.a1 = arg;
        self.registers.sp = sp as usize;
    }
}
