#!/bin/bash

set -e

cd "$(git rev-parse --show-toplevel)"

rust_files="$(git diff --name-only --staged | grep -e '.rs$' || echo -n)"

for file in $rust_files; do
    if [[ -f $file ]]; then
        rustfmt --check $file
    fi
done
