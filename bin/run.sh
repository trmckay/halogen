#!/bin/bash

bin=$(dirname $0)
usage="usage: $0 firmware.bin kernel.bin kernel.elf"

firmware="$1"
kernel="$2"
elf="$3"

qemu="qemu-system-riscv64"

if [[ ! -d "$RISCV_PREFIX" ]]; then
    RISCV_PREFIX="riscv64-unknown-elf-"
fi

qemu_cmd="$qemu -machine virt -cpu rv64 -m 512M -smp 1 -nographic -serial mon:stdio -bios $firmware -s -S -kernel $kernel"
gdb_cmd="rust-gdb -q -ex 'symbol-file $elf' -x $bin/gdbinit"

echo "$qemu_cmd"
echo "$gdb_cmd"

$qemu_cmd &

RUST_GDB=${RISCV_PREFIX}gdb eval $gdb_cmd
