# Pentoxide

An embedded OS for RISC-V made with Rust.


## Run

### Docker

Make sure you have Docker installed and running. Then run `make docker-runner`.

### Baremetal

Make sure you have:

* A Rust toolchain
* `qemu-system-riscv`

Run once: `make init`

Then run: `make run`

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
