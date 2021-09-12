#!/bin/bash

set -e

if ! command -v docker > /dev/null; then
    echo "Warning: cannot find docker."
fi

if ! id -nG "$(whoami)" | grep -qw "docker"; then
    echo "Warning: user is not in the 'docker' group."
fi

if ! command -v rustup > /dev/null; then
    echo "Error: cannot find rustup."
    exit 1
fi

( \
    cd lab-os && \
    rustup override set nightly && \
    rustup target add riscv64gc-unknown-none-elf
)

cat << EOF > .git/hooks/pre-commit
#!/bin/bash

set -e

cd $(git rev-parse --show-toplevel)
make check
EOF

chmod +x .git/hooks/pre-commit
