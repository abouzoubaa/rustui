# Source after changing to the repository root. Cargo otherwise finds rustc
# from PATH, which can differ from rustup's pinned toolchain on developer Macs.
if [ -n "${repo_root:-}" ]; then
  _pinned_root="$repo_root"
elif [ -n "${repository_root:-}" ]; then
  _pinned_root="$repository_root"
else
  _pinned_root="$(pwd)"
fi

_pinned_channel=$(sed -n 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$_pinned_root/rust-toolchain.toml" | sed -n '1p')
if [ -z "$_pinned_channel" ]; then
  printf '%s\n' "error: missing channel in $_pinned_root/rust-toolchain.toml" >&2
  return 1 2>/dev/null || exit 1
fi

# Ignore caller/Homebrew compiler overrides, then explicitly use rustup's
# compiler paths for all rustup invocations in these scripts.
unset RUSTC RUSTDOC
_pinned_rustc_path=$(rustup which rustc --toolchain "$_pinned_channel")
_pinned_rustdoc_path=$(rustup which rustdoc --toolchain "$_pinned_channel")

pinned_rustup() {
  RUSTC="$_pinned_rustc_path" RUSTDOC="$_pinned_rustdoc_path" \
    rustup run "$_pinned_channel" "$@"
}

pinned_cargo() {
  pinned_rustup cargo "$@"
}

pinned_rustc() {
  pinned_rustup rustc "$@"
}
