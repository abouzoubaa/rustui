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

Host audit (2026-10-21): the current host is macOS 27.0 on arm64. A recent
canonical Playground probe produced bounded native keyboard and accessibility-
tree observations (M417), so native macOS validation is possible. A separate
probe built and packaged successfully but could not attach to the app (M418),
so GUI automation availability is not guaranteed. Neither result establishes
WebAssembly support or closes the per-platform X1 scope and acceptance gate.
Evidence: `target/headstart/evidence/sidebar-keyboard-probe-m417-20261021.json`
and `target/headstart/evidence/tree-keyboard-ax-probe-m418-20261021.json`.

## Historical next step at baseline (superseded by refreshed evidence below)

## Refreshed compile viability evidence (2026-10-21)

The earlier failure came from the preinstalled Headstart stage1 sysroot lacking
Wasm `core` and `std`. Built both host and Wasm stage1 libraries from the
matching Headstart Rust source into the repository cache with:

```sh
./x --build-dir /Users/Abium/Developer/rustui/target/headstart/rustc-build \
  build --stage 1 library --target aarch64-apple-darwin,wasm32-unknown-unknown
```

A repo-local launcher root points at that compiler and the existing Headstart
Cargo; the check used a dedicated persistent cache under `target/headstart`:

```sh
CARGO_TARGET_DIR=/Users/Abium/Developer/rustui/target/headstart/x1a-wasm \
RUSTUI_HEADSTART_ROOT=/Users/Abium/Developer/rustui/target/headstart/wasm-root \
  ./scripts/cargo-headstart check -p rustui --target wasm32-unknown-unknown \
    --locked --offline
```

The check passed through the pinned GPUI dependency crates and RustUI. This
establishes compile viability only. It does not establish browser runtime,
packaging, accessibility, or complete WebAssembly platform support. Build and
check logs: `target/headstart/evidence/x1a-host-wasm-std-build-20261021.log`,
`target/headstart/evidence/x1a-wasm-std-build-20261021.log`, and
`target/headstart/evidence/x1a-wasm-rustui-check-20261021.log`.

## Historical next gate at M494 (superseded by M627 update below)

M494 ran `cargo check`, so it type-checked the library and dependencies without
producing a linked browser application. The workspace has no WebAssembly-specific
application entry point or browser packaging/serving harness. The Playground
binary delegates to `rustui_playground::run()`, which currently initializes the
macOS 27 theme. The next concrete gap is therefore an X1b browser-host contract
and smoke application, not another compile-only check.

X1b should first define the supported browser and host contract (canvas/window
creation, event-loop ownership, asset loading, and how JavaScript bindings are
produced and served). Then it should build a linked WebAssembly artifact and
exercise that same artifact in the selected browser, recording startup, visible
rendering, pointer/keyboard input, and accessibility observations. Any claims
must be limited to the browser and behaviors actually exercised. Until those
steps pass, M494 remains compile viability evidence; it does not close X1
platform acceptance. No browser runtime, accessibility, or platform acceptance
was run for this update.

## Update 2026-10-22 — M627 browser entry

M627 adds the Wasm start entry to the canonical Playground, initializes GPUI's browser logging in RustUI's runtime, and selects the Wasm GPUI Kit asset source. The focused target check confirms the Playground binary type-checks. Browser packaging and serving, startup, visible rendering, pointer/keyboard input, asset fetches, and accessibility observations remain unverified. See `docs/tasks/X1B_BROWSER_HOST.md`; X1 platform acceptance stays open until those browser behaviors are exercised.

## Update 2026-10-22 — M629 release bundle generated

The canonical Playground now produces an optimized `wasm-bindgen --target web` bundle in `target/headstart/playground-web`. See `docs/tasks/X1B_BROWSER_HOST.md` and `target/headstart/evidence/m627-stage11-web-release-bundle-build-20261022.log`. Browser startup and behavior remain unverified.
