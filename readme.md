# rVr kernel

Kernel for RISC-V in Rust.

## Requirements

* These packages on Fedora Linux:

    ```bash
    sudo dnf install autoconf automake libmpc-devel mpfr-devel gmp-devel \
        gawk bison flex texinfo patchutils gcc gcc-c++ zlib-devel \
        expat-devel git qemu-system-riscv
    ```

* These packages on Ubuntu Linux:

    ```bash
    sudo apt install autoconf automake autotools-dev curl libmpc-dev \
        libmpfr-dev libgmp-dev gawk build-essential bison flex texinfo gperf \
        libtool patchutils bc zlib1g-dev libexpat-dev git libglib2.0-dev \
        libpixman-1-dev
    ```

* [riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain)

    ```bash
    git clone https://github.com/riscv/riscv-gnu-toolchain
    cd riscv-gnu-toolchain
    ./configure --prefix="$HOME/.local/opt/riscv/rv64gc" --with-arch=rv64gc
    make linux
    ```

* `rustup`

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    rustup default nightly
    rustup toolchain add riscv64gc-unknown-elf
    ```

## Settings for `rust-analyzer`

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