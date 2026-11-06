# M1303 — Stage 9: enforce the string minimum on host edits

## Contract

`PreviewHost::edit` validates string updates against the property's schema
minimum using Unicode character counts. An empty string must be rejected when
the minimum is one, without changing the mounted revision or resolved
properties.

## Change

Added a host regression using a one-character minimum. It attempts an empty
string, confirms `PreviewError::InvalidPropertyValue` for the `text` property,
and verifies that the complete mounted snapshot is unchanged. No production
implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 21/21:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::host_edit_rejects_text_below_schema_min_length --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1303"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::host::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1303"' -- --quiet
```

The local `cc-1.8.0` vendor package omitted four source files. Cargo used a
temporary overlay whose `cc-1.8.0` package came from worktree 6390 after all 36
files listed in its `.cargo-checksum.json` matched. The overlay was removed
after verification; no vendor files were changed and `target/headstart` was
preserved.

This is local inspector schema-enforcement evidence; native and broader
preview acceptance remain separate. D1d composition and D1b IME ownership
decisions remain gated.
