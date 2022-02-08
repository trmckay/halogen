use core::arch::asm;

use crate::{align, arch::satp, mem, mem::paging::pte_flags::*};

pub const EARLY_ALLOC_BLOCK_SIZE: usize = mem::L0_PAGE_SIZE;
pub const EARLY_ALLOC_NUM_BLOCKS: usize = 512; // = 1M
pub const EARLY_ALLOC_SIZE: usize = EARLY_ALLOC_BLOCK_SIZE * EARLY_ALLOC_NUM_BLOCKS;

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

        call early_frame_alloc_init
        call paging_init

        # Expose some global symbols
        .section .data

        .global KERNEL_SIZE
        KERNEL_SIZE: .dword _KERNEL_SIZE

        .global KERNEL_START_VIRT
        KERNEL_START_VIRT: .dword _KERNEL_START_VIRT

        .global KERNEL_START_PHYS
        KERNEL_START_PHYS: .dword 0

        .global TEXT_END
        TEXT_END: .dword _TEXT_END

        .global DATA_END
        DATA_END: .dword _DATA_END

        .global INIT_STACK_TOP
        INIT_STACK_TOP: .dword _INIT_STACK_TOP
    ",
        options(noreturn),
    );
}

/// Get the kernel heap bitmap from its reserved spot.
fn early_frame_alloc_bitmap(
) -> &'static mut mem::Bitmap<EARLY_ALLOC_NUM_BLOCKS, EARLY_ALLOC_BLOCK_SIZE> {
    unsafe {
        ((mem::KERNEL_START_PHYS + mem::MEMORY_SIZE / 2)
            as *mut mem::Bitmap<EARLY_ALLOC_NUM_BLOCKS, EARLY_ALLOC_BLOCK_SIZE>)
            .as_mut()
            .expect("Kernel heap is null")
    }
}

/// Alllocate physical pages from the kernel heap.
pub fn early_frame_alloc() -> Option<*mut u8> {
    let map = early_frame_alloc_bitmap();
    map.alloc(1)
}

/// Initialize a bitmap for pre-paging allocations.
#[no_mangle]
unsafe extern "C" fn early_frame_alloc_init() {
    let bitmap = early_frame_alloc_bitmap();

    // Align to the nearest page, leaving space for the bitmap.
    let bitmap_arena =
        align!(mem::KERNEL_START_PHYS + mem::KERNEL_SIZE, mem::L0_PAGE_SIZE) as *mut u8;

    // Initialize the bitmap.
    *bitmap = mem::Bitmap::new(bitmap_arena);
}

/// Initialize the root page-table and map the kernel.
///
/// TODO: Parse the device tree passed in from OpenSBI.
#[no_mangle]
unsafe extern "C" fn paging_init() -> ! {
    let page_offset = mem::KERNEL_START_VIRT - mem::KERNEL_START_PHYS;
    let text_end_phys = mem::TEXT_END - page_offset;

    // Map the kernel text and data.
    for i in (mem::KERNEL_START_PHYS..text_end_phys).step_by(mem::L0_PAGE_SIZE) {
        mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
            i + page_offset,
            i,
            mem::MappingLevel::FourKilobyte,
            READ | EXECUTE | VALID,
            early_frame_alloc,
        );
    }

    let data_end_phys = mem::DATA_END - page_offset;

    for i in (text_end_phys..data_end_phys).step_by(mem::L0_PAGE_SIZE) {
        mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
            i + page_offset,
            i,
            mem::MappingLevel::FourKilobyte,
            READ | WRITE | VALID,
            early_frame_alloc,
        );
    }

    for i in
        (mem::INIT_STACK_TOP - mem::INIT_STACK_SIZE..mem::INIT_STACK_TOP).step_by(mem::L0_PAGE_SIZE)
    {
        mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
            i,
            i - page_offset,
            mem::MappingLevel::FourKilobyte,
            READ | WRITE | VALID,
            early_frame_alloc,
        );
    }

    // Map the test device.
    mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
        mem::MMIO_DEV_TEST,
        mem::MMIO_DEV_TEST,
        mem::MappingLevel::FourKilobyte,
        READ | WRITE | VALID,
        early_frame_alloc,
    );

    // Map the physical memory.
    // TODO: Map with huge pages.
    for i in (0..mem::MEMORY_SIZE).step_by(mem::L0_PAGE_SIZE) {
        mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
            mem::KERNEL_START_PHYS + i,
            mem::KERNEL_START_PHYS + i,
            mem::MappingLevel::FourKilobyte,
            READ | WRITE | VALID,
            early_frame_alloc,
        );
    }

    // Set the MXR bit.
    asm!("csrc sstatus, {}", in(reg) 1 << 19, options(nostack));

    // Set stvec interrupt vector to kmain.
    let kmain_virt: usize =
        (crate::kmain as usize - mem::KERNEL_START_PHYS) + mem::KERNEL_START_VIRT;

    let free_begin = early_frame_alloc_bitmap().boundary() as usize;

    let gp: usize;
    asm!("mv {}, gp", out(reg) gp);

    asm!("csrw stvec, {}", in(reg) kmain_virt, options(nostack));

    // Write Sv39 config to the SATP register.
    satp::bootstrap(
        satp::Mode::Sv39,
        &mem::ROOT_PAGE_TABLE_RAW,
        0,
        mem::INIT_STACK_TOP,
        gp + page_offset,
        [
            free_begin,
            mem::MEMORY_SIZE - mem::KERNEL_SIZE,
            page_offset,
            0,
            0,
            0,
            0,
            0,
        ],
    );

    // Fence and hope things don't blow up.
    asm!("sfence.vma", options(nostack));
    asm!("nop", options(noreturn))
}
