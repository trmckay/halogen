#!/bin/bash

set -e

ln -svf $(pwd)/bin/pre-commit.sh .git/hooks/pre-commit

if ! command -v rustup > /dev/null; then
    echo "Error: cannot find rustup."
    exit 1
else
    ( \
        cd lab-os && \
        rustup override set nightly && \
        rustup target add riscv64gc-unknown-none-elf
    )
fi

if ! command -v docker > /dev/null; then
    echo "Error: cannot find docker."
    exit 1
else
    if ! id -nG "$(whoami)" | grep -qw "docker"; then
        echo "Error: user is not in the 'docker' group."
        exit 1
    else
        docker pull trmckay/riscv-rv64gc-dev
    fi
fi
