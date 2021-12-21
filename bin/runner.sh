#!/bin/bash

set -e

if ! command -v qemu-system-riscv64 > /dev/null; then
    echo "qemu-system-riscv64 not found"
    exit 1
fi

set -x

if [[ $HALOGEN_DEBUG -eq 1 ]]; then
    qemu-system-riscv64 \
        -machine virt \
        -cpu rv64 \
        -m 256m \
        -smp 1 \
        -nographic \
        -serial mon:stdio \
        -bios none \
        -s -S \
        -kernel $@
else
    qemu-system-riscv64 \
        -machine virt \
        -cpu rv64 \
        -m 256m \
        -smp 1 \
        -nographic \
        -serial mon:stdio \
        -bios none \
        -kernel $@
fi
