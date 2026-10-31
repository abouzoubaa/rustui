# M1308 — Stage 9: enforce the absolute host string byte limit

## Contract

Preview string values are limited to `MAX_STRING_VALUE_BYTES` (16,384 UTF-8
bytes), independent of an optional schema `max_length` character bound.

## Change

Added a host edit regression using a string schema with no `max_length`. It
accepts exactly 16,384 bytes, then rejects 16,385 bytes with
`PreviewError::InvalidPropertyValue` while preserving the complete accepted
snapshot. No production implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 26/26:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::host_edit_rejects_text_beyond_absolute_byte_limit --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::host::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --quiet
```

The local `cc-1.8.0` vendor package omitted four source files. Cargo used a
temporary overlay whose `cc-1.8.0` package came from worktree 6390 after all 36
files listed in its `.cargo-checksum.json` matched. The overlay was removed
after verification; no vendor files were changed and `target/headstart` was
preserved.

This is local host schema-enforcement evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
