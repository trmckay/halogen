#!/bin/bash

# Format or check all source files (intended as a pre-commit hook).

GIT_ROOT=$(git rev-parse --show-toplevel)
rustfmt "$GIT_ROOT"/kernel/src/*.rs
shellcheck "$GIT_ROOT"/scripts/*.sh || exit 1
