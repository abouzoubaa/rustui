#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"
. "$repository_root/scripts/pinned-toolchain.sh"

pinned_cargo fmt --check -p rustui -p rustui-core -p rustui-i18n -p rustui-lab -p rustui-macros -p rustui-playground -p rustui-rgpe
pinned_cargo check --workspace --all-targets --locked --offline
pinned_cargo test --workspace --lib --locked --offline
