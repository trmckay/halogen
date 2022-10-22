use halogen_lib::mem::{Address, PhysicalAddress, Segment};

use crate::{
    io::console::early_println,
    mem::{
        paging::{get_root_satp, map, Permissions, Privilege, Scope, PAGING_ENABLED},
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

    /// Beginning of the kernel image.
    static __text: Symbol;

    /// End of the text section of the kernel image.
    static __text_end: Symbol;

    /// Read-only statics.
    static __ro_data: Symbol;
    /// End of read-only statics.
    static __ro_data_end: Symbol;

    /// Read-write data (e.g. mutable statics).
    static __rw_data: Symbol;
    /// End of read-write data.
    static __rw_data_end: Symbol;

    /// BSS section.
    static __bss: Symbol;
    /// End of BSS section.
    static __bss_end: Symbol;

    /// Read-write region to be used as a stack during boot.
    static __tmp_stack_top: Symbol;

    /// End of the kernel image.
    static __free: Symbol;
}

impl Symbol {
    /// Get the address of a symbol. This uses relative addressing, so will
    /// return physical addresses.
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

/// Initialize the root page table and map the kernel
#[no_mangle]
unsafe extern "C" fn enable_paging(_hart_id: usize, _device_tree: *const u8) -> ! {
    riscv::register::stvec::write(
        early_trap as usize,
        riscv::register::stvec::TrapMode::Direct,
    );

    early_println(BANNER);

    // Calculate and save some constants based on the device-tree (TODO) and linker
    // symbols.

    let virt_offset = KERNEL_SPACE_START.offset(__text.address());

    PHYSICAL_BASE = __text.address();
    PHYSICAL_SIZE = MEMORY_SIZE;

    TEXT_SIZE = __text_end.address() - __text.address();
    RODATA_SIZE = __ro_data_end.address() - __ro_data.address();
    RWDATA_SIZE = __rw_data_end.address() - __rw_data.address();
    FREE_SIZE = MEMORY_SIZE - TEXT_SIZE - RODATA_SIZE - RWDATA_SIZE;

    early_println("\nInitialize frame allocator");

    // Start using the frame allocator initialized at the physical memory just
    // beyond the kernel text and data.
    phys::init(Segment::new(
        __free.address(),
        PHYSICAL_BASE + PHYSICAL_SIZE,
    ));

    early_println("Map kernel image");

    // Map the kernel text.
    map(
        Some(__text.address().add_offset(virt_offset).as_virt()),
        Some(__text.address()),
        TEXT_SIZE,
        Permissions::ReadExecute,
        Scope::Global,
        Privilege::Kernel,
    )
    .unwrap();

    // Map the read-only data.
    map(
        Some(__ro_data.address().add_offset(virt_offset).as_virt()),
        Some(__ro_data.address()),
        RODATA_SIZE,
        Permissions::ReadOnly,
        Scope::Global,
        Privilege::Kernel,
    )
    .unwrap();

    // Map the read-write data.
    map(
        Some(__rw_data.address().add_offset(virt_offset).as_virt()),
        Some(__rw_data.address()),
        RWDATA_SIZE,
        Permissions::ReadWrite,
        Scope::Global,
        Privilege::Kernel,
    )
    .unwrap();

    // Map the rest of the physical memory.
    map(
        Some(__rw_data_end.address().add_offset(virt_offset).as_virt()),
        Some(__rw_data_end.address()),
        FREE_SIZE,
        Permissions::ReadWrite,
        Scope::Global,
        Privilege::Kernel,
    )
    .unwrap();

    // Move the frame allocator to the virtual address-space.
    phys::rebase_virt();

    // This is enough to bootstrap paging.

    early_println("Enable paging\n");

    // Set the MXR bit so executable pages are readable.
    riscv::register::sstatus::set_mxr();

    let satp: usize = get_root_satp();
    let gp: usize = read_reg!(gp) + virt_offset as usize;
    let sp: usize = (__tmp_stack_top.address().add_offset(virt_offset)).into();

    // Set stvec interrupt vector to kmain.
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
