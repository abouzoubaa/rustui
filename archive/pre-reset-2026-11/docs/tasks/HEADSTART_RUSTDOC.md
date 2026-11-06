# Headstart rustdoc and doctests

The RustUI launcher selects Headstart Cargo and the local stage1 rustc at
`~/.local/opt/headstart/rustc/build/aarch64-apple-darwin/stage1/bin/rustc`.
It does not set `RUSTDOC`. The inspected stage1 `bin/` contains only rustc,
which reports `1.101.0-dev` with an unknown commit hash. No `rustdoc` or
`librustdoc` artifact was found in stage1.

The available documentation binaries do not match that compiler:

- Homebrew rustdoc: `1.97.1`.
- Headstart stage0 rustdoc: `1.99.0-beta.3`.
- `build/bootstrap/debug/rustdoc`: a bootstrap shim, which exits without
  `RUSTC_STAGE` and also requires `RUSTDOC_REAL`, `RUSTDOC_LIBDIR`, and
  `RUSTC_SYSROOT`. It is not a standalone matching rustdoc.

The existing Rust bootstrap source registers `src/tools/rustdoc` and
`src/librustdoc` as the rustdoc build target in
`rustc/src/bootstrap/src/core/build_steps/tool.rs`. A supported future route is
to build rustdoc from the same pinned Headstart rustc checkout at the current
stage, then pass its resulting stage1 binary as Cargo `RUSTDOC` alongside the
stage1 rustc. This needs separate toolchain-build authorization and has not
been run here. No toolchain installation, launcher change, cache reset, or
source revision change is implied by this note.

Until a matching rustdoc exists, the launcher can run unit and integration
tests with explicit Cargo target flags (for example `test --lib --tests --bins
--examples --locked`). Those results do not verify doctests. The full test
command's doctest failure with E0514/E0786 reflects the compiler/rustdoc
metadata mismatch, not an accepted documentation test result.
