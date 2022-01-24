use core::{arch::asm, ptr::addr_of};

use crate::{
    align,
    arch::satp,
    mem::{
        paging::{pte_flags::*, *},
        palloc,
        palloc::{get_bitmap, BitmapPageAllocator, PAGE_ALLOC_SIZE},
        stack::KERNEL_STACK_SIZE,
        KERNEL_SIZE, KERNEL_START_PHYS, KERNEL_START_VIRT, MMIO_DEV_TEST_PHYS, MMIO_DEV_TEST_VIRT,
    },
    size_of,
};

/// Entry point called by OpenSBI
/// This should appear at the beginning of the text
#[naked]
#[no_mangle]
#[allow(named_asm_labels)]
#[link_section = ".text.init"]
pub unsafe extern "C" fn _init() -> ! {
    asm!(
        "
        # No compression
        .option norvc

        # Save the physical address where the kernel loads
        auipc a0, 0
        lla a1, KERNEL_START_PHYS
        sd a0, 0(a1)

        lla t0, _KERNEL_START_VIRT

        csrw sie, zero
        csrci sstatus, 2

        # This is a 2M stack to support the trampoline code
        # Can't use the symbol since it's not PI
        lla sp, __init_stack_top
        sub sp, sp, t0    # subtract from linked base to get offset
        add sp, sp, a0    # add offset to load address

        lla t1, __bss_start
        sub t1, t1, t0
        add t1, t1, a0

        lla t2, __bss_end
        sub t2, t2, t0
        add t2, t2, a0

        1:
            beq  t1, t2, 2f
            sd x0, 0(t1)
            addi t1, t1, 8
            j 1b
        2:

        .option push
        .option norelax
        la gp, __global_pointer$
        sub gp, gp, t0
        add gp, gp, a0
        .option pop

        call palloc_init
        j paging_init

        # Expose some global symbols
        .section .data

        .global KERNEL_SIZE
        KERNEL_SIZE: .dword _KERNEL_SIZE

        .global KERNEL_START_VIRT
        KERNEL_START_VIRT: .dword _KERNEL_START_VIRT

        .global KERNEL_START_PHYS
        KERNEL_START_PHYS: .dword 0

        .global PAGING_EN
        PAGING_EN: .dword 0
    ",
        options(noreturn),
    );
}

/// Initialize a single reserved page for use as the kernel
/// heap bitmap
#[no_mangle]
pub unsafe extern "C" fn palloc_init() {
    let root = get_bitmap();

    // Align to the nearest page, leaving space for the bitmap
    let kheap_begin = align!(
        KERNEL_START_PHYS + KERNEL_SIZE + size_of!(BitmapPageAllocator),
        L0_PAGE_SIZE
    );

    // Initialize the bitmap
    *root = BitmapPageAllocator::new(kheap_begin);
}

/// Initialize the root page-table and map the kernel
///
/// TODO: Parse the device tree passed in from OpenSBI
#[no_mangle]
pub unsafe extern "C" fn paging_init() -> ! {
    let page_offset = KERNEL_START_VIRT - KERNEL_START_PHYS;
    let root = PageTable::new();

    // Map the kernel text and data
    for i in (0..KERNEL_SIZE).step_by(L0_PAGE_SIZE) {
        root.map(
            KERNEL_START_VIRT + i,
            KERNEL_START_PHYS + i,
            MappingLevel::FourKilobyte,
            READ | EXECUTE | VALID,
        );
    }

    // Map the page allocator bitmap
    let palloc_bitmap = addr_of!(*palloc::get_bitmap());
    root.map(
        palloc_bitmap as usize + page_offset,
        palloc_bitmap as usize,
        MappingLevel::FourKilobyte,
        READ | WRITE | VALID,
    );

    // Map the page allocator arena
    let palloc_arena = palloc_bitmap.add(L0_PAGE_SIZE);
    for i in (0..L1_PAGE_SIZE).step_by(L0_PAGE_SIZE) {
        root.map(
            palloc_arena as usize + page_offset + i,
            palloc_arena as usize + i,
            MappingLevel::FourKilobyte,
            READ | WRITE | VALID,
        );
    }

    // Map the kernel stack region
    let kstack_bottom = KERNEL_START_PHYS + KERNEL_SIZE + PAGE_ALLOC_SIZE;
    let kstack_top = kstack_bottom + KERNEL_STACK_SIZE;
    for i in (0..KERNEL_STACK_SIZE).step_by(L0_PAGE_SIZE) {
        root.map(
            kstack_bottom + page_offset + i,
            kstack_bottom + i,
            MappingLevel::FourKilobyte,
            READ | WRITE | VALID,
        );
    }

    // Map the test device
    root.map(
        MMIO_DEV_TEST_VIRT,
        MMIO_DEV_TEST_PHYS,
        MappingLevel::FourKilobyte,
        READ | WRITE | VALID,
    );

    // Set the MXR bit
    asm!("csrc sstatus, {}", in(reg) 1 << 19, options(nostack));

    // Set stvec interrupt vector to kmain

    #[cfg(not(test))]
    let kmain_virt: usize = (crate::kmain::kmain as usize - KERNEL_START_PHYS) + KERNEL_START_VIRT;
    #[cfg(test)]
    let kmain_virt: usize =
        (crate::kmain_test::kmain as usize - KERNEL_START_PHYS) + KERNEL_START_VIRT;

    asm!("csrw stvec, {}", in(reg) kmain_virt, options(nostack));

    // Write Sv39 config to the SATP register
    satp::bootstrap(satp::Mode::Sv39, root, 0, kstack_top + page_offset, 0);

    // Fence and hope things don't blow up
    asm!("sfence.vma", options(nostack));
    asm!("nop", options(noreturn))
}
