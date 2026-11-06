# M1310 — Stage 9: enforce inclusive numeric bounds on host edits

## Contract

Finite `Number` values may equal the schema's `minimum` and `maximum`; values
below or above those bounds are rejected without mutating the mounted preview.

## Change

Added a host regression using the numeric fixture's `[0, 1]` range. Edits to
`0.0` and `1.0` succeed and advance the revision. Edits to `-0.1` and `1.1`
return `PreviewError::InvalidPropertyValue` and preserve the complete accepted
snapshot. No production implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 28/28:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::host_edit_accepts_inclusive_number_bounds_and_rejects_outside --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
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
