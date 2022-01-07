# Halogen

[![Build Status](https://drone.trmckay.com/api/badges/tm/halogen/status.svg)](https://drone.trmckay.com/tm/halogen)

A simple OS kernel for RISC-V made with Rust.

## Dev requirements

- Rust with `rustup`

- [`riscv-gnu-toolchain`](https://github.com/riscv-collab/riscv-gnu-toolchain), namely `riscv64-unknown-elf-gdb`
    1. Install build dependencies for your OS
    2. `git clone https://github.com/riscv-collab/riscv-gnu-toolchain`
    3. `./configure --prefix=/path/to/prefix`
    4. `make` (with `sudo` if higher privileges are required to write to the prefix)

- [QEMU](https://gitlab.com/qemu-project/qemu) >= 5 with `qemu-system-riscv64`

- Run `make init` in this repository to configure the Rust toolchain

## Usage

**Run**: `make run`

**Test**: `make test`

### Debugging

Run `make run-debug` or `make test-debug` to launch a GDB server for that task. Attach to the
server with `make attach` in another terminal.

Alternatively, open the repository with VS Code and use the provided `launch.json`.
