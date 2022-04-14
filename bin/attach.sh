#!/bin/bash

elf="$1"
shift 1

if [[ $elf == "" ]]; then
    echo "Usage: $0 /path/to/elf"
    exit 1
fi

if [[ ! -d "$RISCV_PREFIX" ]]; then
    RISCV_PREFIX="riscv64-unknown-elf-"
fi

set -x

export RUST_GDB=${RISCV_PREFIX}gdb
rust-gdb -q \
    -ex "set confirm off" \
    -ex "set architecture riscv:rv64" \
    -ex "symbol-file $elf" \
    -ex "set disassemble-next-line auto" \
    -ex "set riscv use-compressed-breakpoints yes" \
    -ex "target remote:1234" \
    -ex "layout split" \
    -ex "break kinit" \
    -ex "continue" \
