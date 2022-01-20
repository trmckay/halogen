---
title: Boot process
---

# OpenSBI

## Notes

- Runs in machine mode
- Open supervisor binary interface
- Typically used in the boot stage
- Provides some drivers for simple/ubiquitous hardware

See the [OpenSBI documentation](https://github.com/riscv-non-isa/riscv-sbi-doc/blob/master/riscv-sbi.adoc).

## Steps

1. Jumped to by the (usually platform-specific) bootloader.
    + This is only a few instructions at `0x1000` on the QEMU `virt` platform.
2. Does a bunch of platform setup.
3. Loads the payload by jumping to the physical address `0x80200000`.

# Kernel initialization

1. The BSS section is cleared.
2. The global pointer is loaded.

## Kernel heap

1. The kernel heap is 2M of scratch space for the kernel.
2. It is tracked with a bitmap.
3. The page immediately following the kernel text is reserved for the bitmap.
4. The following 2M of pages are used for the arena.

## Paging

1. New page tables are created on the kernel heap.
2. The kernel is identity mapped.
3. The STVEC is written to with the address of `kmain`.
4. The SATP is written and paging switches on.
5. If the kernel traps, it goes to `kmain`. Otherwise, `kmain` is called manually.
