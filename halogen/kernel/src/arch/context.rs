use halogen_lib::{mask, mem::VirtualAddress};

use super::REGISTER_NAMES;
use crate::{mem::regions::Region, read_csr, read_reg};

/// Privilege level that the hart was running at.
#[repr(usize)]
#[derive(Clone, Copy, Debug)]
pub enum Privilege {
    Machine = 3,
    Supervisor = 1,
    User = 0,
}

impl From<Privilege> for usize {
    fn from(prv: Privilege) -> Self {
        match prv {
            Privilege::User => 0,
            Privilege::Supervisor => 1,
            Privilege::Machine => 3,
        }
    }
}

/// Stores everything needed to describe the execution of a task on the CPU.
/// This is not the whole CPU state, but rather the general purpose registers,
/// PC, privilege level, and CSRs for switching address spaces. Other CSRs for
/// things like interrupts are not stored.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Context {
    pub gp_regs: [usize; 31],
    pub pc: usize,
    pub prv: Privilege,
    pub satp: usize,
}

impl core::fmt::Display for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let registers = self.gp_regs;
        let prv = self.prv;

        writeln!(f, "sstatus.spp = {:?}", prv)?;
        writeln!(
            f,
            "sepc = {:p} ({:?})",
            self.pc as *const u8,
            Region::from(VirtualAddress(self.pc))
        )?;

        let satp_mode = (self.satp & mask!(60, 63)) >> 60;
        let satp_asid = (self.satp & mask!(44, 59)) >> 44;
        let satp_ppn = self.satp & mask!(0, 43);

        writeln!(
            f,
            "satp = {{ mode = {}, asid = {}, ppn = 0x{:x} }}",
            satp_mode, satp_asid, satp_ppn
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
        Context {
            gp_regs: [0; 31],
            pc: 0,
            prv: Privilege::User,
            satp: 0,
        }
    }
}

impl Context {
    pub fn new_kernel() -> Context {
        let mut ctx = Context::default();

        ctx.gp_regs[2] = read_reg!(gp);
        ctx.gp_regs[3] = read_reg!(tp);
        ctx.satp = read_csr!(satp);
        ctx.prv = Privilege::Supervisor;

        ctx
    }
}
