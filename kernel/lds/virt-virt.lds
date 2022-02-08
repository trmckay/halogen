OUTPUT_ARCH("riscv")

ENTRY(_init)

SECTIONS
{
    . = 0xC0000000;

    PROVIDE(_KERNEL_START_VIRT = .);

    .text : AT(ADDR(.text)) {
        *(.text.init)
        *(.text .text.*)
    }

    . = ALIGN(4K);

    PROVIDE(_TEXT_END = .);

    .data : AT(ADDR(.data)) {
        *(.data .data.* .rodata .rodata.*)
    }

    .sdata : AT(ADDR(.sdata)) {
        PROVIDE(__global_pointer$ = .);
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);

    PROVIDE(__bss_start = .);

    .sbss : AT(ADDR(.sbss)) {
        *(.sbss .sbss.*);
    }

    .bss : AT(ADDR(.bss)) {
        *(.bss .bss.*)
    }

    . = ALIGN(4K);

    PROVIDE(__bss_end = .);

    .tdata : AT(ADDR(.tdata)) {
        PROVIDE(__tdata_start = .);
        *(.tdata .tdata.*)
        PROVIDE(__tdata_end = .);
    }

    . = ALIGN(4K);

    PROVIDE(_DATA_END = .);
    PROVIDE(_KERNEL_SIZE = . - _KERNEL_START_VIRT);

    . += 4096 * 512;

    .init_stack : AT(ADDR(.init_stack)) {
        PROVIDE(__init_stack_bottom = .);
        . += 1024 * 256;
        PROVIDE(__init_stack_top = .);
    }

    PROVIDE(_INIT_STACK_TOP = .);

    /DISCARD/ : { *(.eh_frame_hdr .eh_frame) }
}
