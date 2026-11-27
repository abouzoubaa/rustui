#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"
. "$repository_root/scripts/pinned-toolchain.sh"
export PATH="$(dirname "$RUSTC"):$PATH"

pinned_cargo clippy --locked --offline --lib --bins \
  -p rustui -p rustui-core -p rustui-i18n -p rustui-macros -p rustui-rgpe -- \
  -D warnings -D clippy::perf \
  -D clippy::redundant_clone -D clippy::needless_collect -D clippy::ptr_arg \
  -D clippy::unnecessary_to_owned -D clippy::inefficient_to_string \
  -D clippy::large_enum_variant -D clippy::trivially_copy_pass_by_ref \
  -D clippy::manual_memcpy -D clippy::map_entry -D clippy::or_fun_call \
  -D clippy::expect_fun_call -D clippy::needless_range_loop
