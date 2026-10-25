# X1a: WebAssembly compile viability

## Result

The first compile-only viability check stopped in dependencies before reaching
RustUI source:

```text
./scripts/cargo-headstart check -p rustui --target wasm32-unknown-unknown --locked --offline
exit code: 101
error[E0463]: can't find crate `core`
error[E0463]: can't find crate `std`
```

The Rustup host installation lists `wasm32-unknown-unknown` as installed, but
`scripts/cargo-headstart` selects the separate Headstart stage1 compiler. Its
sysroot does not contain target libraries under
`/Users/Abium/.local/opt/headstart/rustc/build/aarch64-apple-darwin/stage1/lib/rustlib/wasm32-unknown-unknown/lib`.
Full output: `target/headstart/evidence/x1a-wasm-check-20261019.log`.

## Scope

No source or dependency changes were made. This result does not establish a
RustUI build for WebAssembly, browser packaging, runtime behavior, accessibility,
or platform support.

## Next step

Build the WebAssembly standard library for the Headstart stage1 toolchain or
select an approved compatible toolchain, then rerun the locked, offline check.
If the check passes, decide the browser application and runtime acceptance
scope before claiming WebAssembly support.

## Update 2026-10-25 — Headstart WebAssembly target

Built Stage 1 `wasm32-unknown-unknown` standard-library artifacts from the
existing Headstart source into `target/headstart/x1a-rustc`; the bootstrap
command exited successfully. The source checkout was left unchanged.

The follow-up RustUI check stopped before compiling RustUI:

```text
RUSTC=target/headstart/x1a-rustc/aarch64-apple-darwin/stage1/bin/rustc \
  CARGO_TARGET_DIR=target/headstart ./scripts/cargo-headstart check -p rustui \
  --target wasm32-unknown-unknown --locked --offline
exit code: 101
error: failed to calculate checksum of vendor/cc-1.8.0/src/target/llvm.rs
No such file or directory
```

Exact output: `target/headstart/x1a-rustc/rustui-check.log`. No vendored source,
dependency pin, or lockfile was changed. This does not establish RustUI
WebAssembly compilation. The next step is an approved repair of the incomplete
`cc-1.8.0` vendor snapshot, followed by the same locked check; browser runtime,
accessibility, and platform acceptance remain separate gates.

## Update 2026-10-25 — vendor checksum recovery

Restored the four `cc-1.8.0/src/target/` files listed by its existing checksum
manifest from the matching Cargo-generated vendor snapshot. Their SHA-256
digests match the manifest, and `Cargo.lock` was unchanged. The follow-up
locked/offline WebAssembly check passed checksum validation but stopped while
building host dependencies because the selected Stage 1 compiler lacks its
`aarch64-apple-darwin` `core`/`std` libraries. This still does not establish
RustUI WebAssembly compilation. Next, repair the Stage 1 host sysroot or use the
matching installed Headstart host compiler configuration, then rerun the same
focused check.

## Update 2026-10-25 — Stage 1 host and wasm sysroots

Built Stage 1 host `library/std`, `library/proc_macro`, and `library/test`, then
the `wasm32-unknown-unknown` `library/std`, all through the same Headstart
build directory under `target/headstart/x1a-rustc`. The resulting Stage 1
sysroot contains host and wasm `core`/`std`, plus host `proc_macro`.

The locked, offline `rustui` check then exited successfully:

```sh
RUSTC=target/headstart/x1a-rustc/aarch64-apple-darwin/stage1/bin/rustc \
  CARGO_TARGET_DIR=target/headstart/x1a-rustc \
  ./scripts/cargo-headstart check -p rustui \
  --target wasm32-unknown-unknown --locked --offline
```

The check emitted three existing RustUI warnings (two unused imports and the
unused `ButtonPalette::disabled_text` field). Full output is preserved at
`target/headstart/x1a-rustc/rustui-check.log`. This proves compile-only
WebAssembly viability for the `rustui` library; Playground browser startup,
accessibility, rendering, and platform acceptance remain separate work.
