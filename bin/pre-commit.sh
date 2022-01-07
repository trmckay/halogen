#!/bin/bash

set -e

cd "$(git rev-parse --show-toplevel)"

git stash -q --keep-index

make fmt
git add -u

git stash pop -q
