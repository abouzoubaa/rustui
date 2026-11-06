# M1305 — Stage 9: count Unicode scalars for the host string minimum

## Contract

Preview string `min_length` uses the Unicode scalar count returned by
`str::chars().count()`, not the UTF-8 byte count.

## Change

Added a host edit regression that submits `é` against a minimum of two. The
value is two UTF-8 bytes but one Unicode scalar, so the host rejects it with
`PreviewError::InvalidPropertyValue` and preserves the complete mounted
snapshot. No production implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 23/23:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::host_edit_counts_unicode_scalars_for_schema_min_length --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::host::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --quiet
```

The local `cc-1.8.0` vendor package omitted four source files. Cargo used a
temporary overlay whose `cc-1.8.0` package came from worktree 6390 after all 36
files listed in its `.cargo-checksum.json` matched. The overlay was removed
after verification; no vendor files were changed and `target/headstart` was
preserved.

This is local inspector schema-enforcement evidence; native and broader
preview acceptance remain separate. D1d composition and D1b IME ownership
decisions remain gated.
