# M1304 — Stage 9: count Unicode scalars in host string lengths

## Contract

Preview string `min_length` and `max_length` bounds use Unicode scalar counts,
as implemented by `str::chars().count()`, rather than UTF-8 byte counts.

## Change

Added a host edit regression that accepts 100 copies of `é` against the
fixture's 100-scalar maximum. The value is 200 UTF-8 bytes, so this exercises
the declared character-count contract at the actual `PreviewHost::edit`
boundary. The test verifies the accepted revision and resolved value. No
production implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 22/22:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::host_edit_counts_unicode_scalars_for_schema_max_length --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::host::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --quiet
```

The local `cc-1.8.0` vendor package omitted four source files. Cargo used a
temporary overlay whose `cc-1.8.0` package came from worktree 6390 after all 36
files listed in its `.cargo-checksum.json` matched. The overlay is temporary;
no vendor files were changed and `target/headstart` was preserved.

This is local inspector schema-enforcement evidence; native and broader
preview acceptance remain separate. D1d composition and D1b IME ownership
decisions remain gated.
