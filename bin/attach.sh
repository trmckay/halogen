#!/bin/bash

tmp="$(git rev-parse --show-toplevel)/.halogen-kernel.tmp"
gdb="riscv64-unknown-elf-gdb"

if ! command -v $gdb > /dev/null; then
    echo "$gdb not found"
    exit 1
fi

if ! test -f $tmp; then
    echo "No kernel linked, is the GDB server running?"
    exit 1
fi

echo "target remote localhost:1234" >> .gdbinit

$gdb -q $tmp

rm -f .gdbinit
