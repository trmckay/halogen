#!/bin/bash

GIT_ROOT=$(git rev-parse --show-toplevel)

rustfmt "$GIT_ROOT"/kernel/src/*

shellcheck "$GIT_ROOT"/scripts/*.sh || exit 1
