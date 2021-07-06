/* Based on: https://github.com/sgmarz/osblog/blob/master/risc_v/chapters/ch0/virt.lds */

OUTPUT_ARCH("riscv")

ENTRY(_start)

MEMORY
{
  ROM (rx)   : ORIGIN = 0x1000,     LENGTH = 32K
  RAM (rwxa) : ORIGIN = 0x80000000, LENGTH = 128M
}

PHDRS
{
    text PT_LOAD;
    data PT_LOAD;
    bss  PT_LOAD;
}

SECTIONS
{
    .text : {
        PROVIDE(_text_start = .);
        *(.text.init) *(.text .text.*)
        PROVIDE(_text_end = .);
    } >RAM AT>RAM :text

    PROVIDE(_global_pointer = .);

    .rodata : {
        PROVIDE(_rodata_start = .);
        *(.rodata .rodata.*)
        PROVIDE(_rodata_end = .);
    } >RAM AT>RAM :text

    .data : {
        . = ALIGN(4096);
        PROVIDE(_data_start = .);
        *(.sdata .sdata.*) *(.data .data.*)
        PROVIDE(_data_end = .);
    } >RAM AT>RAM :data

    .bss : {
        PROVIDE(_bss_start = .);
        *(.sbss .sbss.*) *(.bss .bss.*)
        PROVIDE(_bss_end = .);
    } >RAM AT>RAM :bss

    PROVIDE(_memory_start = ORIGIN(RAM));
    PROVIDE(_stack_start = _bss_end);
    PROVIDE(_stack_end = _stack_start + 0x80000);
    PROVIDE(_memory_end = ORIGIN(RAM) + LENGTH(RAM));
    PROVIDE(_heap_start = _stack_end);
    PROVIDE(_heap_size = _memory_end - _heap_start);
}
