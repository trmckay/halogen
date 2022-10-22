# Booting with OpenSBI

Clone OpenSBI v1.1 from [riscv-software-src/opensbi](https://github.com/riscv-software-src/opensbi).

```bash
$ git clone --single-branch -b v1.1 https://github.com/riscv-software-src/opensbi
```

Build the firmware for target platform (`generic` for QEMU).

```bash
$ make -j$(nproc) FW_PIC=no PLATFORM=generic
```

The firmware will be located at `build/platform/$PLATFORM/firmware/fw_jump.bin`. To boot the kernel
using QEMU, use the `run-qemu` script.

```bash
$ scripts/run-qemu fw_jump.bin halogen-debug.bin
```
