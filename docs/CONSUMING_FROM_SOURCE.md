# Consuming RustUI from a source checkout

This is a development example for an independent Rust application using the
public `rustui` API from a complete RustUI source checkout. It is not a
crates.io recipe or a public release; RustUI packages remain `publish = false`
and release acceptance is incomplete. The currently declared target boundary is
Apple Silicon macOS and Web/Wasm; this example does not certify runtime support.

## Run the example in the checkout

The standalone workspace is in `examples/external-consumer/`. Its `Cargo.toml`
points to the SDK's `crates/rustui` and local `vendor/` sources. From the SDK
checkout root, run:

```sh
./scripts/cargo-headstart run \
  --manifest-path examples/external-consumer/Cargo.toml \
  --target-dir target/headstart/external-consumer
```

This requires the already provisioned Headstart toolchain described in
[`docs/BUILDING.md`](BUILDING.md); the launcher does not install it. Build
artifacts stay in the SDK's `target/headstart/` tree.

The example activates the SDK's bundled theme during `on_launch`, before its
components render. Its `include_str!` path to `themes/macos27.json` must also
be adjusted if the example is moved outside this checkout.

The example has an independent `[workspace]` root. It repeats the approved
same-version local patches because Cargo workspace `[patch.crates-io]` entries
are not inherited from the SDK when consumed by an external root. It also copies
the SDK's scoped `rustui-macros` incremental settings because profile settings
are not inherited across workspaces. Keep these entries aligned with the SDK's
root `Cargo.toml` and approved patch policy in [`patches/README.md`](../patches/README.md).
The example imports and uses only `rustui` public API; it does not add direct
application dependencies on GPUI or GPUI Components.

## Copying this example outside the checkout

The checked-in-relative paths deliberately target this SDK checkout. If copying
the app elsewhere, change the `rustui` path and each patch path to the
corresponding paths in the complete SDK checkout, or place the app as a member
of a chosen source workspace and retain that workspace root's explicit patch
entries. Keep the approved vendor sources and patch series with the SDK. Do not
replace these path patches with unreviewed registry versions or infer that
`cargo publish` is supported.

This documents only current source-checkout consumption. It does not define a
git-dependency or archive distribution contract, certify additional platforms,
or imply publication, signing, or release approval.
