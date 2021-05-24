#!/bin/bash

# Build a binary then create/open an object dump.

SAVED_DIR=$(pwd)
GIT_ROOT=$(git rev-parse --show-toplevel)
OBJDUMP="riscv64-unknown-linux-gnu-objdump"
BINARY="$GIT_ROOT/kernel/target/riscv64gc-unknown-none-elf/debug/rvr-kernel"

cd "$GIT_ROOT"/kernel || exit 1
cargo build

cd "$SAVED_DIR" || exit 1

if (( $# < 1 )); then
    "$OBJDUMP" -S "$BINARY" | less
else
    "$OBJDUMP" -S "$BINARY" > "$1"
fi