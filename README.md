# Halogen

Halogen is an operating system kernel for RISC-V rv64gc. It's called Halogen
because it's lightweight and blows up if you aren't careful.

## Development

### Requirements

-   [`rustup`](https://rustup.rs) or Rust with `riscv64gc-unknown-none-elf` target
-   [`riscv64-unknown-elf-` toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)
-   [`qemu-system-riscv64`](https://www.qemu.org)

### Usage

[`cargo-xtask`](https://github.com/matklad/cargo-xtask/) is used as the build system, There
are workflows defined to build, run, test, debug, etc. Run `cargo xtask help`.

## Related

-   [OpenSBI](https://github.com/riscv-software-src/opensbi)
-   [docker-rust-riscv64](https://github.com/trmckay/docker-rust-riscv64)
