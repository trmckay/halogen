use core::{arch, ptr};

use riscv::register;

use crate::{
    align,
    irq::plic,
    mem,
    mem::{paging::pte_flags::*, Allocator, StaticBitmap},
};

extern "C" {
    pub static KERNEL_SIZE: usize;
    pub static KERNEL_START_PHYS: usize;
    pub static KERNEL_START_VIRT: usize;
    pub static TEXT_END: usize;
    pub static DATA_END: usize;
    pub static INIT_STACK_TOP: usize;
}

const EARLY_ALLOC_BLOCK_SIZE: usize = mem::L0_PAGE_SIZE;
const EARLY_ALLOC_NUM_BLOCKS: usize = 512; // = 1M
const EARLY_ALLOC_SIZE: usize = EARLY_ALLOC_BLOCK_SIZE * EARLY_ALLOC_NUM_BLOCKS;

const INIT_STACK_SIZE: usize = 256 * 1024;

#[naked]
#[no_mangle]
#[allow(named_asm_labels)]
#[link_section = ".text.init"]
unsafe extern "C" fn boot() -> ! {
    arch::asm!(
        "
        # No compression
        .option norvc

        # Save the physical address where the kernel loads
        auipc a0, 0
        lla a1, KERNEL_START_PHYS
        sd a0, 0(a1)

        lla t0, _KERNEL_START_VIRT

        # Disable supervisor interrupts for now
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

        call bootstrap

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

/// Get the kernel heap bitmap from its reserved spot
fn early_frame_alloc_bitmap(
) -> &'static mut StaticBitmap<EARLY_ALLOC_NUM_BLOCKS, EARLY_ALLOC_BLOCK_SIZE> {
    unsafe {
        ((KERNEL_START_PHYS + mem::MEMORY_SIZE / 2)
            as *mut StaticBitmap<EARLY_ALLOC_NUM_BLOCKS, EARLY_ALLOC_BLOCK_SIZE>)
            .as_mut()
            .expect("Kernel heap is null")
    }
}

/// Alllocate physical pages from the kernel heap
pub fn early_frame_alloc() -> Option<usize> {
    let map = early_frame_alloc_bitmap();
    match map.alloc::<[u8; mem::paging::L0_PAGE_SIZE]>(1) {
        Err(_) => None,
        Ok(ptr) => Some(ptr as usize),
    }
}

/// Initialize a bitmap for pre-paging allocations
#[no_mangle]
unsafe extern "C" fn early_frame_alloc_init() {
    let bitmap = early_frame_alloc_bitmap();

    // Align to the nearest page, leaving space for the bitmap
    let bitmap_arena = align!(KERNEL_START_PHYS + KERNEL_SIZE, mem::L0_PAGE_SIZE) as *mut u8;

    // Initialize the bitmap
    *bitmap = StaticBitmap::new(bitmap_arena);
}

/// Initialize the root page-table and map the kernel
///
/// TODO: Parse the device tree passed in from OpenSBI
#[no_mangle]
unsafe extern "C" fn bootstrap() -> ! {
    early_frame_alloc_init();

    let page_offset = KERNEL_START_VIRT - KERNEL_START_PHYS;
    let text_end_phys = TEXT_END - page_offset;

    // Map the kernel text and data
    for boot in (KERNEL_START_PHYS..text_end_phys).step_by(mem::L0_PAGE_SIZE) {
        mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
            boot + page_offset,
            boot,
            mem::MappingLevel::FourKilobyte,
            READ | EXECUTE | VALID,
            early_frame_alloc,
        );
    }

    let data_end_phys = DATA_END - page_offset;

    for addr in (text_end_phys..data_end_phys).step_by(mem::L0_PAGE_SIZE) {
        mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
            addr + page_offset,
            addr,
            mem::MappingLevel::FourKilobyte,
            READ | WRITE | VALID,
            early_frame_alloc,
        );
    }

    for addr in (INIT_STACK_TOP - INIT_STACK_SIZE..INIT_STACK_TOP).step_by(mem::L0_PAGE_SIZE) {
        mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
            addr,
            addr - page_offset,
            mem::MappingLevel::FourKilobyte,
            READ | WRITE | VALID,
            early_frame_alloc,
        );
    }

    // Map the test device.
    mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
        mem::DEV_TEST + mem::MMIO_OFFSET,
        mem::DEV_TEST,
        mem::MappingLevel::FourKilobyte,
        READ | WRITE | VALID,
        early_frame_alloc,
    );

    // Map the PLIC
    for addr in 0..=2 {
        mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
            mem::DEV_PLIC + mem::MMIO_OFFSET + (addr * mem::L0_PAGE_SIZE),
            mem::DEV_PLIC + (addr * mem::L0_PAGE_SIZE),
            mem::MappingLevel::FourKilobyte,
            READ | WRITE | VALID,
            early_frame_alloc,
        );
    }

    // Map the PLIC context
    // TODO: Map one context (two pages) for each hart
    let num_harts = 1;
    for hart in 0..num_harts {
        for i in 0..=1 {
            let hart_offset = plic::PLIC_CTX_HART_STEP * hart;
            mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
                mem::DEV_PLIC_CONTEXT + mem::MMIO_OFFSET + hart_offset + (mem::L0_PAGE_SIZE * i),
                mem::DEV_PLIC_CONTEXT + hart_offset + (mem::L0_PAGE_SIZE * i),
                mem::MappingLevel::FourKilobyte,
                READ | WRITE | VALID,
                early_frame_alloc,
            );
        }
    }

    // Map the UART device
    mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
        mem::DEV_UART + mem::MMIO_OFFSET,
        mem::DEV_UART,
        mem::MappingLevel::FourKilobyte,
        READ | WRITE | VALID,
        early_frame_alloc,
    );

    // Map the physical memory
    // TODO: Map with huge pages
    for i in (0..mem::MEMORY_SIZE).step_by(mem::L0_PAGE_SIZE) {
        mem::ROOT_PAGE_TABLE_RAW.map_with_allocator(
            KERNEL_START_PHYS + i,
            KERNEL_START_PHYS + i,
            mem::MappingLevel::FourKilobyte,
            READ | WRITE | VALID,
            early_frame_alloc,
        );
    }

    // Set the MXR bit
    register::sstatus::set_mxr();

    // Set stvec interrupt vector to kmain
    let kinit: usize = (crate::kinit as usize - KERNEL_START_PHYS) + KERNEL_START_VIRT;

    let free_begin = early_frame_alloc_bitmap().boundary() as usize;

    let gp: usize;
    arch::asm!("mv {}, gp", out(reg) gp);

    // Write Sv39 config to the SATP register.
    satp_bootstrap(
        register::satp::Mode::Sv39,
        &mem::ROOT_PAGE_TABLE_RAW,
        0,
        INIT_STACK_TOP,
        gp + page_offset,
        kinit,
        [
            free_begin,
            mem::MEMORY_SIZE - KERNEL_SIZE,
            page_offset,
            0,
            0,
            0,
            0,
            0,
        ],
    );

    // Fence and hope things don't blow up
    riscv::asm::sfence_vma_all();
    arch::asm!("unimp", options(noreturn));
}

/// Set the paging mode and bounce to an entry-point
unsafe fn satp_bootstrap(
    mode: register::satp::Mode,
    root: &mem::PageTable,
    asid: usize,
    sp: usize,
    gp: usize,
    entry: usize,
    args: [usize; 8],
) {
    register::stvec::write(entry, register::mtvec::TrapMode::Direct);
    let mode: u64 = match mode {
        register::satp::Mode::Bare => 0,
        register::satp::Mode::Sv39 => 8,
        _ => unimplemented!("paging mode not implemented"),
    };
    let ppn = ptr::addr_of!(*root) as usize >> 12;
    let satp: usize = ((mode as usize) << 60) | (asid << 44) | ppn;
    arch::asm!("
        mv sp, {}
        mv gp, {}
        csrw satp, {}
    ", in(reg) sp, in(reg) gp, in(reg) satp,
       inout("a0") args[0] => _,
       inout("a1") args[1] => _,
       inout("a2") args[2] => _,
       inout("a3") args[3] => _,
       inout("a4") args[4] => _,
       inout("a5") args[5] => _,
       inout("a6") args[6] => _,
       inout("a7") args[7] => _
    )
}
