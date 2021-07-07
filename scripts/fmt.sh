#!/bin/bash

# Format or check all source files (intended as a pre-commit hook).

GIT_ROOT=$(git rev-parse --show-toplevel)

shopt -s globstar
for file in "$GIT_ROOT"/**/*.rs; do
    rustfmt --check "$file"
done

shopt -s globstar
for file in "$GIT_ROOT"/**/*.sh; do
    shellcheck "$file"
done
