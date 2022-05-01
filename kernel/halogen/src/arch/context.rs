use crate::{
    prelude::*,
    thread::{ThreadFunction, ThreadShim},
};

#[repr(usize)] // `usize` to simplify alignment
#[derive(Clone, Copy, Debug)]
pub enum Privilege {
    Machine = 0,
    Supervisor = 1,
    User = 2,
}


// Some fields are pointers so they are debug-printed with pointer format
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GpRegisters {
    ra: *const u8,
    sp: *const u8,
    gp: *const u8,
    tp: *const u8,
    t0: usize,
    t1: usize,
    t2: usize,
    fp: *const u8,
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

unsafe impl Sync for GpRegisters {}
unsafe impl Send for GpRegisters {}

impl Default for GpRegisters {
    fn default() -> GpRegisters {
        unsafe {
            GpRegisters {
                ra: transmute(0_usize),
                sp: transmute(0_usize),
                gp: transmute(0_usize),
                tp: transmute(0_usize),
                t0: 0,
                t1: 0,
                t2: 0,
                fp: transmute(0_usize),
                s1: 0,
                a0: 0,
                a1: 0,
                a2: 0,
                a3: 0,
                a4: 0,
                a5: 0,
                a6: 0,
                a7: 0,
                s2: 0,
                s3: 0,
                s4: 0,
                s5: 0,
                s6: 0,
                s7: 0,
                s8: 0,
                s9: 0,
                s10: 0,
                s11: 0,
                t3: 0,
                t4: 0,
                t5: 0,
                t6: 0,
            }
        }
    }
}

/// Stores a CPU state for interrupts/traps, context switches, etc.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub registers: GpRegisters,
    pub pc: *const u8,
    pub prv: Privilege,
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Default for Context {
    /// Create a new context from kernel code
    fn default() -> Context {
        let mut ctx = Context {
            registers: GpRegisters::default(),
            pc: ptr::null(),
            prv: Privilege::Supervisor,
        };

        ctx.registers.gp = read_reg!(gp) as *const _;
        ctx.registers.tp = read_reg!(tp) as *const _;

        ctx
    }
}

impl Context {
    pub fn prepare(&mut self, sp: *mut u8, shim: ThreadShim, entry: ThreadFunction, arg: usize) {
        self.pc = shim as *const _;
        self.registers.a0 = entry as usize;
        self.registers.a1 = arg;
        self.registers.sp = sp;
    }
}
