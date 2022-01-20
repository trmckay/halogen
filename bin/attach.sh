#!/bin/bash

elf="$1"

if [[ $elf == "" ]]; then
    "Usage: $0 /path/to/elf"
    exit 1
fi

if [[ ! -f $elf ]]; then
    make $elf
fi

if [[ ! -d "$RISCV_PREFIX" ]]; then
    RISCV_PREFIX="riscv64-unknown-elf-"
fi

set -x

${RISCV_PREFIX}gdb -q $elf
