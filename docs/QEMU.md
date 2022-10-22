# Running with QEMU

Currently, the `qemu-system-riscv64` `virt` machine is the only supported platform. A
convenience script is provided in `scripts/run-qemu`. The following variables can be set
to tune the behavior.

| Variable   | Default               | Purpose                |
| ---------- | --------------------- | ---------------------- |
| `QEMU`     | `qemu-system-riscv64` | Path to QEMU           |
| `QEMU_SMP` | `1`                   | Number of cores        |
| `QEMU_MEM` | `512`                 | Amount of memory in MB |

Usage: `scripts/run-qemu [bios] kernel`
