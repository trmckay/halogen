# rVr kernel

Kernel for RISC-V in Rust.

## Requirements

* `qemu-system-riscv`

* [riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain)

    ```bash
    git clone https://github.com/riscv/riscv-gnu-toolchain
    cd riscv-gnu-toolchain
    ./configure --prefix="$HOME/.local/opt/riscv/rv64gc" --with-arch=rv64gc
    make linux
    ```
    
* Rust

## Setup

```
git clone git@github.com:trmckay/rVr-kernel.git
cd rVr-kernel
rustup override set nightly
rustup target add riscv64gc-unknown-none-elf
```

### Extra settings for `rust-analyzer`

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
