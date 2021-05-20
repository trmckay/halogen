#!/bin/bash

GIT_ROOT=$(git rev-parse --show-toplevel)

export PATH="$PATH:$GIT_ROOT/scripts"
