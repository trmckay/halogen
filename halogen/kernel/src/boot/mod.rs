use halogen_common::mem::{Address, PhysicalAddress, Segment};

use crate::{
    io::console::early_println,
    mem::{
        paging::{map, root_satp, Permissions, PAGING_ENABLED},
        phys,
        regions::{
            FREE_SIZE, KERNEL_SPACE_START, PHYSICAL_BASE, PHYSICAL_SIZE, RODATA_SIZE, RWDATA_SIZE,
            TEXT_SIZE,
        },
        MEMORY_SIZE,
    },
    read_reg,
    trap::early_trap,
};

const BANNER: &str = r"
 _           _
| |__   __ _| | ___   __ _  ___ _ __
| '_ \ / _` | |/ _ \ / _` |/ _ \ '_ \
| | | | (_| | | (_) | (_| |  __/ | | |
|_| |_|\__,_|_|\___/ \__, |\___|_| |_|
                     |___/";

extern "C" {
    type Symbol;

    static __text: Symbol;
    static __text_end: Symbol;

    static __ro_data: Symbol;
    static __ro_data_end: Symbol;

    static __rw_data: Symbol;
    static __rw_data_end: Symbol;

    static __bss: Symbol;
    static __bss_end: Symbol;

    static __tmp_stack_top: Symbol;

    static __free: Symbol;
}

impl Symbol {
    /// Get the address of a symbol
    ///
    /// This uses relative addressing, so will yield physical addresses
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress(self as *const Symbol as usize)
    }
}

#[naked]
#[no_mangle]
#[allow(named_asm_labels)]
#[link_section = ".text.init"]
unsafe extern "C" fn entry() -> ! {
    core::arch::asm!(include_str!("entry.s"), options(noreturn));
}

/// Initialize the root page-table and map the kernel
#[no_mangle]
unsafe extern "C" fn enable_paging(_hart_id: usize, _device_tree: *const u8) -> ! {
    riscv::register::stvec::write(
        early_trap as usize,
        riscv::register::stvec::TrapMode::Direct,
    );

    early_println(BANNER);

    // Calculate and save some constants based on the device-tree (TODO) and linker
    // symbols

    let virt_offset = KERNEL_SPACE_START.offset_from(__text.address());

    PHYSICAL_BASE = __text.address();
    PHYSICAL_SIZE = MEMORY_SIZE;

    TEXT_SIZE = __text_end.address() - __text.address();
    RODATA_SIZE = __ro_data_end.address() - __ro_data.address();
    RWDATA_SIZE = __rw_data_end.address() - __rw_data.address();
    FREE_SIZE = MEMORY_SIZE - TEXT_SIZE - RODATA_SIZE - RWDATA_SIZE;

    early_println("\nInitialize frame allocator");

    // Start using the frame allocator initialized at the physical memory just
    // beyond the kernel text and data
    phys::init(Segment::new(
        __free.address(),
        PHYSICAL_BASE + PHYSICAL_SIZE,
    ));

    early_println("Map kernel image");

    // Map the kernel text
    map(
        Some(__text.address().add_offset(virt_offset as isize).as_virt()),
        Some(__text.address()),
        TEXT_SIZE,
        Permissions::ReadExecute,
    )
    .unwrap();

    // Map the read-only data
    map(
        Some(
            __ro_data
                .address()
                .add_offset(virt_offset as isize)
                .as_virt(),
        ),
        Some(__ro_data.address()),
        RODATA_SIZE,
        Permissions::ReadOnly,
    )
    .unwrap();

    // Map the read-write data
    map(
        Some(
            __rw_data
                .address()
                .add_offset(virt_offset as isize)
                .as_virt(),
        ),
        Some(__rw_data.address()),
        RWDATA_SIZE,
        Permissions::ReadWrite,
    )
    .unwrap();

    // Map the rest of the physical memory
    map(
        Some(
            __rw_data_end
                .address()
                .add_offset(virt_offset as isize)
                .as_virt(),
        ),
        Some(__rw_data_end.address()),
        FREE_SIZE,
        Permissions::ReadWrite,
    )
    .unwrap();

    // Move the frame allocator to the virtual address-space
    phys::rebase_virt();

    // This is enough to bootstrap paging

    early_println("Enable paging\n");

    // Set the MXR bit
    riscv::register::sstatus::set_mxr();

    let satp: usize = root_satp();
    let gp: usize = read_reg!(gp) + virt_offset as usize;
    let sp: usize = (__tmp_stack_top.address().add_offset(virt_offset as isize)).into();

    // Set stvec interrupt vector to kmain
    let kinit = crate::kinit as usize + virt_offset as usize;
    riscv::register::stvec::write(kinit, riscv::register::mtvec::TrapMode::Direct);

    PAGING_ENABLED = true;

    core::arch::asm!(
        "mv sp, {}",
        "mv gp, {}",
        "csrw satp, {}",
        "sfence.vma zero, zero",
        "unimp",
        in(reg) sp,
        in(reg) gp,
        in(reg) satp,
        options(noreturn)
    );
}
