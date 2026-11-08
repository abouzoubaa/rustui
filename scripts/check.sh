#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"

rustup run 1.97.1 cargo fmt --check -p rustui -p rustui-core -p rustui-i18n -p rustui-lab -p rustui-macros -p rustui-playground -p rustui-rgpe
rustup run 1.97.1 cargo check --workspace --all-targets --locked --offline
rustup run 1.97.1 cargo test --workspace --lib --locked --offline
