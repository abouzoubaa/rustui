# M1316 — Stage 9: validate integer defaults during registry registration

## Contract

An integer property default must satisfy its schema bounds. If a default is
outside those bounds, `PreviewRegistry::register` rejects the component without
occupying its ID; a corrected schema with that ID remains registerable and
mountable.

## Change

Added a registry regression that rejects default `5` with maximum `4`, checks
the registry remains empty, then registers the same component with default `4`
and verifies the mounted value and render path. No production implementation
change was needed.

## Verification

The focused locked/offline test passed 1/1, and the complete
`preview::registry::tests` module passed 4/4. Full command output is recorded
in `target/headstart/evidence/m1316-preview-registry-invalid-integer-default-20261024.log`.

```sh
./scripts/cargo-headstart test -p rustui --lib preview::registry::tests::out_of_range_integer_default_keeps_registry_available_for_corrected_schema --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::registry::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --quiet
```

The temporary vendor overlay used a replacement `cc-1.8.0` package from
worktree 6390 after all 36 manifest-listed files matched their checksums. The
overlay symlinks were removed afterward; no vendor files were changed and
`target/headstart` was preserved.

This is local registry and host-mount evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
