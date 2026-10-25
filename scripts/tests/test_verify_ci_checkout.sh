#!/bin/sh
set -eu
repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
fixture=$(mktemp -d "${TMPDIR:-/tmp}/rustui-ci-checkout.XXXXXX")
trap 'rm -rf "$fixture"' EXIT HUP INT TERM
cd "$fixture"
git init -q
git config user.name Fixture
git config user.email fixture@example.invalid
printf 'target/\n' > .gitignore
printf 'original\n' > source.rs
git add .gitignore source.rs
git -c commit.gpgsign=false commit -qm fixture
revision=$(git rev-parse HEAD)
mkdir -p target/headstart
printf 'persistent\n' > target/headstart/cache
"$repository_root/scripts/verify-ci-checkout.sh" "$revision"
printf 'stale\n' > stale.rs
if "$repository_root/scripts/verify-ci-checkout.sh" "$revision" >/dev/null 2>&1; then
    exit 1
fi
rm stale.rs
if "$repository_root/scripts/verify-ci-checkout.sh" 0000000000000000000000000000000000000000 >/dev/null 2>&1; then
    exit 1
fi
printf 'changed\n' > source.rs
if "$repository_root/scripts/verify-ci-checkout.sh" "$revision" >/dev/null 2>&1; then
    exit 1
fi
test "$(cat target/headstart/cache)" = persistent
printf '%s\n' 'CI checkout guard rejects stale sources, tracked changes and the wrong revision while retaining caches.'
