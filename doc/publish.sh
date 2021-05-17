#!/bin/bash

PWD=$(pwd)
GIT_ROOT=$(git rev-parse --show-toplevel)

echo Updating using Neuron Docker.
echo Close when done.

cd "$GIT_ROOT/doc" || exit 1
rm -rf .neuron
docker-compose up

echo "Publishing changes to gh-pages branch."
cd "$GIT_ROOT" || exit 1
git subtree push --prefix "$GIT_ROOT/doc/.neuron/output" origin gh-pages

cd "$PWD" || exit 1
