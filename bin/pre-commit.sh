#!/bin/bash

set -e

cd "$(git rev-parse --show-toplevel)"

rust_files="$(git diff --name-only --staged | grep -e '.rs$' || echo -n)"

if [ -n "$rust_files" ]; then
    rustfmt --check -v "$rust_files"
fi
