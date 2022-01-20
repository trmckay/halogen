#!/bin/bash

kernel="build/halogen.bin"
firmware="build/fw.bin"
qemu="qemu-system-riscv64"
debug_flags=""

if [[ $# -gt 2 ]]; then
    echo "Usage: $0 [-g] /path/to/kernel"
    exit 1
elif [[ $1 == "-g" ]]; then
    debug_flags="-s -S"
fi

if [[ ! -f $kernel ]]; then
    make $kernel
fi
if [[ ! -f $firmware ]]; then
    make $firmware
fi

echo "Launching $qemu"
exec $qemu \
    -machine virt \
    -cpu rv64 \
    -m 512M \
    -smp 1 \
    -nographic \
    -serial mon:stdio \
    -bios $firmware \
    $debug_flags \
    -kernel $kernel
