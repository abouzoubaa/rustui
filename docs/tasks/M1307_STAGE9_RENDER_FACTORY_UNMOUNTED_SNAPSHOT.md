# M1307 — Stage 9: capture the unmounted render-factory state

## Contract

`PreviewHost::render_factory()` captures an immutable render description. A
factory captured while the host is unmounted continues to describe that empty
state after a later mount; a newly captured factory describes the mounted
preview.

## Change

Added a host regression that captures a render factory before mount, confirms
it returns `None` both before and after mounting, and confirms a new factory
renders the mounted preview. No production implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 25/25:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::render_factory_keeps_unmounted_snapshot_after_later_mount --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::host::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --quiet
```

The local `cc-1.8.0` vendor package omitted four source files. Cargo used a
temporary overlay whose `cc-1.8.0` package came from worktree 6390 after all 36
files listed in its `.cargo-checksum.json` matched. The overlay was removed
after verification; no vendor files were changed and `target/headstart` was
preserved.

This is source-level render-factory evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
