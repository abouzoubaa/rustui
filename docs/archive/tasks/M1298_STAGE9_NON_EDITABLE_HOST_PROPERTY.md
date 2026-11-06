# M1298 — Stage 9: reject inspector edits to non-editable properties

## Contract

`ValidatedPreview::apply_batch` rejects updates to properties whose schema has
`editable: false`. `PreviewHost::edit` exposes that validation to inspector
updates and must leave the current mounted snapshot intact on rejection.

## Change

Added a host regression using the existing text fixture with editability turned
off. A typed text edit returns `PreviewError::NotEditable` and preserves the
complete pre-edit snapshot. No production implementation change was needed.
This covers the inspector edit path only; it does not define owner binding,
native text-input, selection, composition, or IME behavior.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::non_editable_property_rejects_host_edit_without_mutating_snapshot --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1295"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` package was incomplete. Cargo used a
temporary overlay combining the existing vendor entries with the package from
worktree 6390 after all 36 files listed in its `.cargo-checksum.json` matched.
The overlay was removed after the test; no vendor files were changed and the
target cache was not cleaned.

Native and broader preview acceptance remain separate. D1d composition and
D1b IME ownership decisions remain gated.
