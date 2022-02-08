---
title: Boot process
---

# OpenSBI

- Runs in machine mode
- Open supervisor binary interface
- Typically used in the boot stage
- Provides some drivers for simple/ubiquitous hardware

See the [OpenSBI documentation](https://github.com/riscv-non-isa/riscv-sbi-doc/blob/master/riscv-sbi.adoc).

1. Jumped to by the (usually platform-specific) bootloader.
2. Does a bunch of platform setup.
3. Loads the payload by jumping to the physical address `0x80200000`.

# Kernel initialization

1. The BSS section is cleared.
2. The global and stack pointers are loaded.
3. An early allocator page allocator is initialized right after the kernel text.
4. The kernel address space is mapped to the higher half (`0xC000000`).
5. The `stvec` register is populated with the virtual address of `kmain()`.
6. The `satp` register is bootstrapped along with the new `sp`/`gp`.
7. On the first invalid fetch, the CPU traps to the `stvec`.
