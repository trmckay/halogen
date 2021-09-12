#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CpuFrame {
    pub registers: [usize; 31],
    pub float_registers: [usize; 32],
    pub satp: usize,
    pub stack_frame: *mut u8,
    pub hartid: usize,
}
