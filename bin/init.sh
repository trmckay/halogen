#!/bin/bash

echo -e "\nThis script will:"
echo -e "\t* Install the required Rust toolchain."
echo -e "\t* Install pre-commit hooks."

set -e

ln -svf "$(pwd)"/bin/pre-commit.sh .git/hooks/pre-commit

if ! command -v rustup > /dev/null; then
    echo "Error: cannot find rustup."
    exit 1
else
    ( \
        cd kernel && \
        rustup override set nightly && \
        rustup target add riscv64gc-unknown-none-elf && \
        rustup component add rustfmt --toolchain nightly-x86_64-unknown-linux-gnu
    )
fi
