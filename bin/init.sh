#!/bin/bash

set -e

read -p "Install pre-commit (pip3) and set-up hooks? [Y/n] " ans

if [[ "$ans" == "y" || "$ans" == "Y" ]]; then
    pip3 install pre-commit
    pre-commit install
fi

if ! grep -q "add-auto-load-safe-path $(pwd)/.gdbinit" "$HOME/.gdbinit" 2> /dev/null; then
    echo
    read -p "Allow gdbinit to auto-load at this path for automatic attach? [y/N] " ans

    if [[ "$ans" == "y" || "$ans" == "Y" ]]; then
        set -x
        echo  "add-auto-load-safe-path $(pwd)/.gdbinit" >> "$HOME/.gdbinit"
        set +x
        echo
    fi
fi
