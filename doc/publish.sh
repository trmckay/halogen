#!/bin/bash

PWD=$(pwd)
GIT_ROOT=$(git rev-parse --show-toplevel)

set -x
set -e

echo Updating using Neuron Docker.
echo Close when done.

cd "$GIT_ROOT/doc" || exit 1
rm -rf .neuron
docker-compose up

echo "Publishing changes to gh-pages branch."
cd "$GIT_ROOT" || exit 1
git subtree push --prefix "doc/.neuron/output" origin gh-pages

cd "$PWD" || exit 1
