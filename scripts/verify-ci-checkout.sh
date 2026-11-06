#!/bin/sh
set -eu

expected_sha=${1:?usage: verify-ci-checkout.sh EXPECTED_SHA}
if [ "$(git rev-parse HEAD)" != "$expected_sha" ]; then
    printf '%s\n' 'Release checkout does not match the selected revision.' >&2
    exit 1
fi
if [ -n "$(git status --porcelain=v1 --untracked-files=all)" ]; then
    git status --short
    printf '%s\n' 'Release checkout contains tracked changes or untracked source files.' >&2
    exit 1
fi
