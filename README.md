# Halogen

[![Build status](https://drone.trmckay.com/api/badges/tm/halogen/status.svg)](https://drone.trmckay.com/tm/halogen)

A simple OS kernel for RISC-V made with Rust.


## Development

### Requirements

- [`rustup`](https://rustup.rs) or Rust with `riscv64gc-unknown-none-elf` target
- [`riscv64-unknown-elf-` toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)
- [`qemu-system-riscv64`](https://www.qemu.org)

### Usage

[`cargo-xtask`](https://github.com/matklad/cargo-xtask/) is used as the build system, since
Cargo alone cannot apply any post-processing to compiler artifacts.

```
$ cargo xtask --help
Halogen cargo-xtask build system

USAGE:
    xtask <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    build    Build the kernel in ./build
    check    Cargo check/clippy/fmt each crate
    clean    Clean up compiler artifacts
    fmt      Check format with rustfmt
    help     Print this message or the help of the given subcommand(s)
    run      Run the kernel in QEMU
    test     Run unit tests in QEMU

```

## Related

- [docker-rust-riscv64](https://git.trmckay.com/tm/docker-rust-riscv64)
- [riscv-dev-ansible](https://git.trmckay.com/tm/riscv-dev-ansible)
