SECTIONS
{
    PROVIDE(__sp = 0x80000000);

    . = 0x1000;

    .text : {
        *(.text.init)
        *(.text)
    }

    . = ALIGN(4K);

    .rodata : { *(.rodata) }

    . = ALIGN(4K);

    PROVIDE(__global_pointer$ = .);

    .data : { *(.data) }
    .bss : { *(.bss) }

    . = ALIGN(4K);
}
