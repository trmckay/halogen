# Pentoxide

An embedded OS for RISC-V made with Rust.

## Clone and configure

```
git clone git@github.com:trmckay/pentoxide.git
cd pentoxide
make init
```

## Run

### Docker

Make sure you have Docker installed and running. Then run `make run-docker`.

### Baremetal

Make sure you have:

* A Rust toolchain
* `qemu-system-riscv`

Then run: `make run`.

## Documentation

The preferred method for building uses Docker, use it with `make docs`.

## Extras

### Settings for `rust-analyzer`

`settings.json`:
```json
{
    "rust-analyzer.checkonsave.alltargets": false,
    "rust-analyzer.checkonsave.extraargs": [
        "--target",
        "riscv64gc-unknown-none-elf"
    ]
}
```
