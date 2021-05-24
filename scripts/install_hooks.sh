#!/bin/bash

# Install pre-commit hooks.

GIT_ROOT="$(git rev-parse --show-toplevel)"
ln -svf "$GIT_ROOT"/scripts/pre-commit.sh "$GIT_ROOT"/.git/hooks/pre-commit
