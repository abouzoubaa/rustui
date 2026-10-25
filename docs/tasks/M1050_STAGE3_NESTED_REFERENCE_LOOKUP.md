# M1050 — resolve nested Playground reference assets

Playground reference lookup now supports both the canonical flattened asset
catalog and packaged reference trees that preserve their subdirectories. It
tries the existing flat filename first, then strips the catalog's
`docs/ui/macos-27/` prefix and resolves the remaining nested path. This matches
the packager, which copies complete directory trees.

The two focused flat and nested lookup regressions passed 2/2 with:

```text
./scripts/cargo-headstart test -p rustui-playground --lib nested_catalog_reference_resolves
```

Native GUI and packaged-app launch acceptance remain open.
