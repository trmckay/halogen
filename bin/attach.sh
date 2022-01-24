#!/bin/bash

elf="$1"

if [[ $elf == "" ]]; then
    echo "Usage: $0 /path/to/elf"
    exit 1
fi

if [[ ! -d "$RISCV_PREFIX" ]]; then
    RISCV_PREFIX="riscv64-unknown-elf-"
fi


set -x

${RISCV_PREFIX}gdb \
    -ex "target remote:1234" \
    -ex "break kmain" \
    -q $elf
