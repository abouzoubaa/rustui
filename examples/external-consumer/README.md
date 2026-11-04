# Local source consumer

This example builds a small app using RustUI's public API. The setup below
creates a separate Cargo workspace outside the checkout, but it still uses the
checkout's RustUI crates, pinned vendor sources, patches, theme and Headstart
toolchain. It is a local development workflow, not an installed package.

From the RustUI checkout, choose a **new** destination whose parent exists:

```sh
RUSTUI_CHECKOUT="$PWD"
python3 examples/external-consumer/create-local.py "$HOME/rustui-counter"
cd "$HOME/rustui-counter"
"$RUSTUI_CHECKOUT/scripts/cargo-headstart" build --profile headstart --locked --offline
```

The generator refuses an existing destination. It copies the example lockfile
and theme and configures absolute paths to this checkout; moving or removing
the checkout breaks those paths. A failed setup can leave a partially created
destination for inspection. The launcher requires the approved Headstart
installation and normally reuses the checkout's `target/headstart` cache.

The build produces `target/headstart/headstart/rustui-external-consumer-example`
under the checkout unless `CARGO_TARGET_DIR` selects another target. It does
not establish packaged distribution, signing, license clearance or release
acceptance. See [release verification](../../docs/RELEASE.md) for those gates.
