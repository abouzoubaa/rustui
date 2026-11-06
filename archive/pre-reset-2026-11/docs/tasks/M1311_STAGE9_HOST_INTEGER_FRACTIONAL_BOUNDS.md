# M1311 — Stage 9: apply fractional schema bounds to host integers

## Contract

An integer property constrained by fractional `minimum` and `maximum` bounds
accepts only integers within the effective ceiling and floor of those bounds.
Rejected host edits preserve the mounted preview.

## Change

Added a host regression with schema bounds `1.1..=4.9`. Integer values `2` and
`4` are accepted; `1` and `5` return
`PreviewError::InvalidPropertyValue`. The complete accepted snapshot remains
unchanged after both rejected edits. No production implementation change was
needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 29/29:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::host_edit_applies_fractional_bounds_to_integer_values --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
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
