# M1309 — Stage 9: reject non-finite host number edits

## Contract

Preview `Number` values must be finite. A rejected host edit must preserve the
mounted revision and resolved property values.

## Change

Added a numeric host fixture and a regression that submits NaN, positive
infinity, and negative infinity. Each update returns
`PreviewError::InvalidPropertyValue`, and the complete mounted snapshot remains
unchanged after every rejection. No production implementation change was
needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 27/27:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::host_edit_rejects_non_finite_numbers_atomically --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
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
