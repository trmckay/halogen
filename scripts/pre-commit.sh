#!/bin/bash

# Run pre-commit hooks.

set -e

GIT_ROOT=$(git rev-parse --show-toplevel)

fail=0

echo -n "Running format checks... "
if ! "$GIT_ROOT"/scripts/fmt.sh; then
    echo "failed."
    fail=$((fail+1))
else
    echo "passed."
fi

echo -n "Running build checks... "
cd "$GIT_ROOT" || exit 1
if ! make build > /dev/null; then
    echo "failed."
    fail=$((fail+1))
else
    echo "passed."
fi

exit $fail
