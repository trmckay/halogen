#!/bin/bash

set -e

ln -svf "$(pwd)"/bin/pre-commit.sh .git/hooks/pre-commit

if [[ $NONINTERACTIVE -ne 1 ]] && ! grep -q "add-auto-load-safe-path $(pwd)/.gdbinit" "$HOME/.gdbinit" 2> /dev/null; then
    echo
    read -p "Allow gdbinit to auto-load at this path for automatic attach? [y/N] " ans

    if [[ "$ans" == "y" || "$ans" == "Y" ]]; then
        set -x
        echo  "add-auto-load-safe-path $(pwd)/.gdbinit" >> "$HOME/.gdbinit"
        set +x
        echo
    fi
fi

if [[ $NONINTERACTIVE -ne 1 ]]; then
    echo
    read -p "Install drone-cli to ~/.local/bin to run CI locally? [y/N]" ans

    if [[ "$ans" == "y" || "$ans" == "Y" ]]; then
        url="https://github.com/harness/drone-cli/releases/latest/download/drone_linux_amd64.tar.gz"

        pwd=$(pwd)
        tmp=$(mktemp -d)
        cd $tmp

        wget -q $url
        tar xvf $(basename $url) > /dev/null
        mv -v drone ~/.local/bin

        cd $pwd
        rm -r $tmp
    fi
fi

if ! command -v rustup > /dev/null; then
    echo "Error: cannot find rustup."
    exit 1
else
    set -x
    cd kernel
    rustup override set nightly
    rustup target add riscv64gc-unknown-none-elf
    rustup component add rustfmt --toolchain nightly-x86_64-unknown-linux-gnu
    set +x
fi
