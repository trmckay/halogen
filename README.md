# Halogen

[![Build Status](https://drone.trmckay.com/api/badges/tm/halogen/status.svg)](https://drone.trmckay.com/tm/halogen)

A simple OS kernel for RISC-V made with Rust.

## Dev requirements

- `rustup`
- `riscv64-unknown-elf-gdb`
- `qemu-system-riscv64`

To setup the repository, run `make init` and respond to the prompts.

## Usage

Run `make run` or `make test`.

Run `make run-debug` or `make test-debug` to launch a GDB server for that task. Attach to the
server with `make attach` in another terminal.

Alternatively, open the repository with VS Code and use the provided `launch.json` and `tasks.json`.

## External resources

- Crate documentation at `https://static.trmckay.com/halogen/<branch>/halogen`
- [rustup](https://rustup.rs)
- [riscv-gnu-toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)
- [docker-rust-riscv64](https://git.trmckay.com/tm/docker-rust-riscv64)
- [Drone](https://drone.io)
- [pre-commit](https://pre-commit.com)
