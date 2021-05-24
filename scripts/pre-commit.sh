#!/bin/bash

# Run pre-commit hooks.

GIT_ROOT=$(git rev-parse --show-toplevel)
"$GIT_ROOT"/scripts/fmt.sh
