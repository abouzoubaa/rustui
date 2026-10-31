# M1302 — Stage 9: enforce the string maximum on host edits

## Contract

`PreviewHost::edit` validates string updates against the property's schema
maximum. A value longer than the configured maximum must be rejected without
changing the mounted revision or resolved properties.

## Change

Added a host regression using the existing 100-character text property. It
submits 101 ASCII characters, confirms `PreviewError::InvalidPropertyValue`
for the `text` property, and verifies that the complete mounted snapshot is
unchanged. No production implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 20/20:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::host_edit_rejects_text_beyond_schema_max_length --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1295"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::host::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1295"' -- --quiet
```

The reconstructed checkout's `cc-1.8.0` package was incomplete. Cargo used a
temporary overlay combining the existing vendor entries with the package from
worktree 6390 after all 36 files listed in its `.cargo-checksum.json` matched.
The overlay was removed after the tests; no vendor files were changed and the
target cache was not cleaned.

This is local inspector schema-enforcement evidence; native and broader
preview acceptance remain separate. D1d composition and D1b IME ownership
decisions remain gated.
