---
title: Building
---

# Dependencies

## OpenSBI

OpenSBI can be compiled with the `riscv64-unknown-elf-` toolchain. It's packaged in
Ubuntu under `gcc-riscv64-unknown-elf` and `binutils-riscv64-unknown-elf`. Many other
distributions package it as well.

It can also be built from [source](https://github.com/riscv-collab/riscv-gnu-toolchain).
The simplest way to get a working toolchain is this:

1. Install the build dependencies
2. `configure --prefix /path/to/prefix --enable-multilib`
3. `[sudo] make`

## Kernel

The main dependency for the kernel is the Rust toolchain manager, `rustup`. It can be
installed from [rustup.rs](https://rustup.rs/).

To build the test, `jq` is required. Cargo emits its test binaries to a non-deterministic
path. Optionally, Cargo logs to JSON; this is how Make locates the test binary.

# Instructions

Assuming you have the dependencies, running `make` will create all the build artifacts
to the `build` directory. This includes the firmware image, a kernel image, a kernel
ELF (for debugging symbols), and an object dump of the whole text (for both main and test).

# Process

See the [`Makefile`][../Makefile] for details on how these are created. In short:

1. A firmware image is created with OpenSBI's `Makefile`
2. Cargo compiles the kernel and links it to a base address of `0x8020_0000` (this is an ELF)
3. The ELF is stripped and dumped to a binary image

QEMU can then load the OpenSBI firmware image as the BIOS. Once it's setup is complete, it
jumps to `0x8020_0000`, where the kernel is loaded.

# Notes

## Linking and loading

The base address for the linker script and base address into which the kernel is loaded
are **not** necessarily the same. If the link address differs from the load address, make
sure all the code preceding paging is position-independent.
