# M1312 — Stage 9: handle the minimum i64 host bound

## Contract

Integer schema bounds at `i64::MIN` remain representable through the numeric
schema representation. A value above an inclusive maximum of `i64::MIN` must
be rejected without changing the mounted preview.

## Change

Added a host regression with both integer bounds set to `i64::MIN as f64` and
that value as the default. Mount accepts the single-value range; editing to
`i64::MIN + 1` returns `PreviewError::InvalidPropertyValue` and preserves the
complete mounted snapshot. No production implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 30/30:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::host_mount_and_edit_handle_i64_minimum_bound --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
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
