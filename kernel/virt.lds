OUTPUT_ARCH("riscv")

ENTRY(_init)

SECTIONS
{
    . = 0x80200000;

    PROVIDE(__text_begin = .);

    .text : AT(ADDR(.text)) {
        PROVIDE(KERNEL_ENTRY = .);
        *(.text.init)
        *(.text .text.*)
        . = ALIGN(4K);
    }

    PROVIDE(__text_end = .);
    PROVIDE(__data_start = .);

    .data : AT(ADDR(.data)) {
        *(.data .data.* .rodata .rodata.*)
    }

    .sdata : AT(ADDR(.sdata)) {
        PROVIDE(__global_pointer$ = .);
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);

    PROVIDE(__data_end = .);
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
        . = ALIGN(4K);
        PROVIDE(__tdata_start = .);

        *(.tdata .tdata.*)

        . = ALIGN(4K);
        PROVIDE(__tdata_end = .);
    }

    .init_stack : AT(ADDR(.init_stack)) {
        PROVIDE(__init_stack_bottom = .);
        . = ALIGN(4K);
        . += 1024 * 1024 * 4;
        . = ALIGN(4K);
        PROVIDE(__init_stack_top = .);
    }

    PROVIDE(_KERNEL_END = .);

    /DISCARD/ : { *(.eh_frame_hdr .eh_frame) }
}
