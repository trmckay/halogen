# rVr kernel

Kernel for RISC-V in Rust.

## Requirements

* Rust

* `qemu-system-riscv`

* [riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain)

    ```bash
    git clone https://github.com/riscv/riscv-gnu-toolchain
    cd riscv-gnu-toolchain
    ./configure --prefix="$HOME/.local/opt/riscv/rv64gc" --with-arch=rv64gc
    make linux
    ```

### Build `qemu-system-riscv` on Ubuntu

```bash
sudo apt install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
    gawk build-essential bison flex texinfo gperf libtool patchutils bc \
    zlib1g-dev libexpat-dev git libpixman-1-dev libglib2.0-dev
git clone https://github.com/qemu/qemu
cd qemu
git checkout v5.0.0
./configure --target-list=riscv64-softmmu
make -j $(nproc)
sudo make install
```

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
