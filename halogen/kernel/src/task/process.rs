use alloc::vec::Vec;

use super::{loader::load_elf, thread::UserThread};
use crate::{error::KernelResult, mem::AddressSpace};

#[derive(Debug, Clone, Default)]
pub struct Process {
    pub pid: usize,
    pub space: AddressSpace,
    pub main_tid: usize,
    pub tids: Vec<usize>,
}

impl Process {
    pub fn try_from_elf(pid: usize, elf: &[u8]) -> KernelResult<Process> {
        let mut space = AddressSpace::new(pid);
        load_elf(&mut space, elf)?;

        Ok(Process {
            pid,
            space,
            main_tid: 0,
            tids: Vec::default(),
        })
    }

    pub fn create_main(&mut self, tid: usize) -> KernelResult<UserThread> {
        self.main_tid = tid;
        let thread = UserThread::try_new(tid, self)?;
        Ok(thread)
    }
}
