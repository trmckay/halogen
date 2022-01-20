use core::arch::asm;

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

        lla sp, __init_stack_top

        lla t0, __bss_start
        lla t1, __bss_end

        1:
            beq  t0, t1, 2f
            sd x0, 0(t0)
            addi t0, t0, 8
            j 1b
        2:

        .option push
        .option norelax
        la gp, __global_pointer$
        .option pop

        # Initialize the kernel heap
        addi sp, sp, -4
        sd a0, 0(sp)
        call kmalloc_init
        ld a0, 0(sp)
        addi sp, sp, 4

        # a1 <- end of the kernel text
        lla a1, _KERNEL_END

        j paging_init

        # Expose some linker symbols
        .section .data
        .global KERNEL_END
        KERNEL_END: .dword _KERNEL_END
    ",
        options(noreturn),
    );
}
