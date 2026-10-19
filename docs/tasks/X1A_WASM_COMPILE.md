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
