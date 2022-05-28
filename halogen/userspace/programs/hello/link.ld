SECTIONS {

    . = 0x1000;

    .text : AT(ADDR(.text)) {
        *(.text.init)
        *(.text .text.*)
    }

    . = ALIGN(4K);

    .rodata : AT(ADDR(.rodata)) {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4K);

    .data : AT(ADDR(.data)) {
        *(.data .data.*)
    }

    .sdata : AT(ADDR(.sdata)) {
        PROVIDE(__global_pointer$ = .);
        *(.sdata .sdata.*)
    }

    .sbss : AT(ADDR(.sbss)) {
        *(.sbss .sbss.*);
    }

    .bss : AT(ADDR(.bss)) {
        *(.bss .bss.*)
    }

    .tdata : AT(ADDR(.tdata)) {
        *(.tdata .tdata.*)
    }

    . = ALIGN(4K);
}
