# Halogen

Halogen is an operating system kernel for RISC-V rv64gc. It's called Halogen
because it's lightweight and blows up if you aren't careful.

## Development

### Requirements

- [`rustup`](https://rustup.rs) or Rust with `riscv64gc-unknown-none-elf` target
- [`riscv64-unknown-elf-` toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)
- [`qemu-system-riscv64`](https://www.qemu.org)

### Usage

[`cargo-xtask`](https://github.com/matklad/cargo-xtask/) is used as the build system, since
Cargo alone cannot apply any post-processing to compiler artifacts. Run `cargo xtask --help`
for a list of build tasks.

## Related

- [OpenSBI](https://github.com/riscv-software-src/opensbi)
- [docker-rust-riscv64](https://github.com/trmckay/docker-rust-riscv64)
- [riscv-dev-ansible](https://github.com/trmckay/riscv-dev-ansible)
