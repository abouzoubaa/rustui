# M1315 — Stage 9: preserve host state after a rejected edit batch

## Contract

`PreviewHost::edit` applies a typed property batch atomically. If a batch stages
a valid edit and then encounters a property that is not declared by the
component, the mounted preview and its revision remain unchanged.

## Change

Strengthened the existing host regression for a batch containing a valid text
update followed by an undeclared property: it now compares the complete
mounted snapshot with the pre-edit snapshot. No production implementation
change was needed.

## Verification

The focused locked/offline test passed 1/1, and the complete
`preview::host::tests` module passed 30/30. Full command output is recorded in
`target/headstart/evidence/m1315-preview-host-atomic-batch-20261024.log`.

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::valid_update_before_unknown_property_keeps_current_preview_unchanged --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::host::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --quiet
```

The temporary vendor overlay used a replacement `cc-1.8.0` package from
worktree 6390 after all 36 manifest-listed files matched their checksums. The
overlay symlinks were removed afterward; no vendor files were changed and
`target/headstart` was preserved.

This is local host-edit atomicity evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
