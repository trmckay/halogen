#!/bin/bash

SAVED_DIR=$(pwd)
GIT_ROOT=$(git rev-parse --show-toplevel)
OBJDUMP="riscv64-unknown-linux-gnu-objdump"
BINARY="$GIT_ROOT/kernel/target/riscv64gc-unknown-none-elf/debug/rvr-kernel"

if (( $# < 1 )); then
    echo "Destination file required."
    exit 1
fi

cd "$GIT_ROOT"/kernel || exit 1
cargo build

cd "$SAVED_DIR" || exit 1
"$OBJDUMP" -S -s "$BINARY" > "$1"
