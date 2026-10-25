# M1301 — Stage 9: enforce allowed string values on host edits

## Contract

For a string property with `allowed_values`, `PreviewHost::edit` accepts only a
listed value. A rejected inspector update must preserve the mounted revision
and resolved property values.

## Change

Added a host fixture with `hello` and `world` as the allowed text values. The
regression first edits to `world`, then attempts `other`; the second edit
returns `PreviewError::InvalidPropertyValue` for the property and leaves the
complete accepted snapshot unchanged. No production implementation change was
needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 19/19:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::host_edit_rejects_values_outside_the_allowed_set --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1295"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::host::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1295"' -- --quiet
```

The reconstructed checkout's `cc-1.8.0` package was incomplete. Cargo used a
temporary overlay combining the existing vendor entries with the package from
worktree 6390 after all 36 files listed in its `.cargo-checksum.json` matched.
The overlay was removed after the tests; no vendor files were changed and the
target cache was not cleaned.

This is local inspector schema-enforcement evidence; native and broader
preview acceptance remain separate. D1d composition and D1b IME ownership
decisions remain gated.
