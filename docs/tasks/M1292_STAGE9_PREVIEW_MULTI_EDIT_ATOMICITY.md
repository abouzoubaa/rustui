# M1292 — Stage 9: reject mixed-validity preview edits atomically

## Contract

`PreviewHost::edit` applies a typed property batch atomically to the exact
current stamp. If a later update is invalid, earlier valid updates in that
batch must not change the mounted preview or its revision.

## Change

Added a host-level regression that submits a valid text update followed by an
unknown property. The batch is rejected, and the mounted stamp and resolved
text remain unchanged. This covers host-level rollback for a multi-property
failure path, in addition to the schema-level batch test and duplicate-ID host
regression. No production implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::valid_update_before_unknown_property_keeps_current_preview_unchanged --locked --offline --config 'source.vendored-sources.directory="/var/folders/t3/sszszn6s4xl0q_5x3rr0c_800000gn/T/rustui-m1292-vendor-kspflmof"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` vendor package is incomplete. Cargo
used a temporary source overlay combining the existing vendor entries with
the checksum-verified package from worktree 6390. The overlay was removed
afterward; no vendor files were changed and the target cache was not cleaned.

This is source-level edit-atomicity evidence; broader native preview acceptance
remains separate. D1d composition and D1b IME ownership decisions remain
gated.
