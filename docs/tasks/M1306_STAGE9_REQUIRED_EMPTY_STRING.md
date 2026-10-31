# M1306 — Stage 9: keep required separate from non-empty strings

## Contract

`required` means the property must be supplied. A supplied empty string remains
valid when the schema has no `min_length`; non-empty constraints are expressed
by the separate string-length bound.

## Change

Added a host mount regression that supplies an empty string for a required text
property with no default and no minimum length. Mount succeeds and the resolved
snapshot contains the supplied empty value. No production implementation
change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 24/24:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::required_empty_string_is_accepted_without_min_length --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::host::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --quiet
```

The local `cc-1.8.0` vendor package omitted four source files. Cargo used a
temporary overlay whose `cc-1.8.0` package came from worktree 6390 after all 36
files listed in its `.cargo-checksum.json` matched. The overlay was removed
after verification; no vendor files were changed and `target/headstart` was
preserved.

This is local descriptor-validation evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
