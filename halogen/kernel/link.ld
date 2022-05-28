OUTPUT_ARCH("riscv")

ENTRY(entry)

SECTIONS
{
    /* Upper region of a 39-bit address-space */
    . = 0xFFFFFFC000000000;

    /***************************************/
    /* Kernel text                         */

    PROVIDE(__text = .);

    .text : AT(ADDR(.text)) {
        *(.text.init)
        *(.text .text.*)
    }

    . = ALIGN(4K);

    PROVIDE(__text_end = .);

    /***************************************/
    /* Read-only data                      */

    PROVIDE(__ro_data = .);

    .rodata : AT(ADDR(.rodata)) {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4K);

    PROVIDE(__ro_data_end = .);

    /***************************************/
    /* Read-write data                     */

    PROVIDE(__rw_data = .);

    .data : AT(ADDR(.data)) {
        *(.data .data.*)
    }

    .sdata : AT(ADDR(.sdata)) {
        PROVIDE(__global_pointer$ = .);
        *(.sdata .sdata.*)
    }

    PROVIDE(__bss = .);

    .sbss : AT(ADDR(.sbss)) {
        *(.sbss .sbss.*);
    }

    .bss : AT(ADDR(.bss)) {
        *(.bss .bss.*)
    }

    PROVIDE(__bss_end = .);

    .tdata : AT(ADDR(.tdata)) {
        *(.tdata .tdata.*)
    }

    .tmp_stack : AT(ADDR(.tmp_stack)) {
        PROVIDE(__tmp_stack_base = .);
        . += 1024 * 256;
        PROVIDE(__tmp_stack_top = .);
    }

    . = ALIGN(4K);

    PROVIDE(__rw_data_end = .);
    PROVIDE(__free = .);

    /***************************************/

    /DISCARD/ : { *(.eh_frame_hdr .eh_frame) }
}
