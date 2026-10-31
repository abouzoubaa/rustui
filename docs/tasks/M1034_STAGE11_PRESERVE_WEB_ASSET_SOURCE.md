# M1034 — Preserve delegated Web asset sources

## Scope

The Playground wraps GPUI Kit's asset source with a compile-time embedded PNG.
The existing regression used an empty source, so it only verified the embedded
path and did not protect delegation of unrelated assets from the wrapped source.

## Change

The focused test now supplies a small base asset source. It verifies that the
embedded PNG loads, an unrelated base asset still loads, missing paths remain
missing, and the asset listing contains both the delegated and embedded paths.

## Verification

`./scripts/cargo-headstart test -p rustui embedded_asset_resolves_png_bytes_and_path --lib --locked --offline`

Result: 1 passed, 0 failed.

This is source-level regression coverage. Browser asset fetching, rendering,
input, accessibility, and broader Stage 11 acceptance remain open.
