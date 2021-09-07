OUTPUT_ARCH("riscv")

ENTRY(_start)

MEMORY
{
    /* virt machine main memory */
    ram (wxa): ORIGIN = 0x80000000, LENGTH = 256M
}

PHDRS
{
    text PT_LOAD;
    data PT_LOAD;
    bss  PT_LOAD;
}

SECTIONS
{
    /* text section */
    .text : {
        PROVIDE(_text_start = .);
        *(.text.init) *(.text .text.*)
        PROVIDE(_text_end = .);
    } >ram AT>ram :text

    /* section for global variables */
    PROVIDE(_global_pointer = .);


    /* just put read-only data in the text */
    .rodata : {
        PROVIDE(_rodata_start = .);
        *(.rodata .rodata.*)
        PROVIDE(_rodata_end = .);
    } >ram AT>ram :text

    .data : {
        /* sv39 uses 4096 B pages */
        . = ALIGN(4096);
        PROVIDE(_data_start = .);
        /* no distinction necessary for data vs. sdata */
        *(.sdata .sdata.*) *(.data .data.*)
        PROVIDE(_data_end = .);
    } >ram AT>ram :data

    .bss : {
        PROVIDE(_bss_start = .);
        *(.sbss .sbss.*) *(.bss .bss.*)
        PROVIDE(_bss_end = .);
    } >ram AT>ram :bss

    /* memory begins here, not necessarily at zero */
    PROVIDE(_MEM = ORIGIN(ram));
    PROVIDE(_MEM_END = ORIGIN(ram) + LENGTH(ram));

    /* allocate a statically sized chunk for kernel heap */
    PROVIDE(_K_STACK = _bss_end);
    PROVIDE(_K_STACK_END = _K_STACK + 0x80000);

    /* we can use the rest for kernel heap */
    PROVIDE(_K_HEAP = _K_STACK_END);
    PROVIDE(_K_HEAP_SIZE = _MEM_END - _K_HEAP);
}
