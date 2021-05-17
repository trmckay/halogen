#!/bin/bash

set -e

PWD=$(pwd)
GIT_ROOT=$(git rev-parse --show-toplevel)

echo "Commit changes in $GIT_ROOT/doc? [Y/n]"
read -n1 yn
case $yn in
    [Nn]* ) exit;;
    * ) git commit "$GIT_ROOT/doc"
esac

echo Updating using Neuron Docker.
echo Close when done.

cd "$GIT_ROOT/doc" || exit 1
rm -rf .neuron
docker-compose up

echo "Publishing changes to gh-pages branch."
cd "$GIT_ROOT" || exit 1
git subtree push --prefix "doc/.neuron/output" origin gh-pages

cd "$PWD" || exit 1
