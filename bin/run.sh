#!/bin/bash

binary="$1"
tmp="$(git rev-parse --show-toplevel)/.halogen-kernel.tmp"
qemu="qemu-system-riscv64"

if [[ $# -gt 1 ]]; then
    echo "Too many arguments"
    exit 1
fi

if [[ "$binary" == "" ]]; then
    echo "No binary supplied"
    exit 1
fi

if [[ $(echo "$binary" | head -c 1) != "/" ]]; then
    binary="$(pwd)/$binary"
fi

echo "Launching $binary with $qemu"

if ! command -v $qemu > /dev/null; then
    echo "$qemu not found"
    exit 1
fi

if test -f $tmp; then
    echo "GDB server already running"
    exit 1
fi

ln -sf "$binary" "$tmp"

if [[ $HALOGEN_DEBUG -eq 1 ]]; then
    $qemu \
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
    $qemu \
        -machine virt \
        -cpu rv64 \
        -m 256m \
        -smp 1 \
        -nographic \
        -serial mon:stdio \
        -bios none \
        -kernel $@
fi

status=$?

rm -f $tmp

exit $status