# Halogen

Halogen is an operating system kernel for RISC-V rv64gc. It's called Halogen
because it's lightweight and blows up if you aren't careful.

## Development

### Requirements

- [`rustup`](https://rustup.rs) or Rust with `riscv64gc-unknown-none-elf` target
- [`riscv64-unknown-elf-` toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)
- [`qemu-system-riscv64`](https://www.qemu.org)
- [`just`](https://github.com/casey/just)

### Usage

Build a recipe with the `just` command.

```bash
$ just --list    # List recipes
$ just [recipe]  # Build a recipe
```

## More documentation

See the [`docs`](./docs/README.md)
