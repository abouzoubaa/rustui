# M1313 — Stage 9: cover the upper i64 boundary in the preview registry

## Contract

An integer schema whose minimum is `2^63` has no representable `i64` value and
must be rejected without registering its component. The registry must remain
available for a corrected schema with maximum `2^63` and default `i64::MAX`;
mounting resolves that default and rendering succeeds.

## Change

Added a `PreviewRegistry` regression for atomic rejection and recovery at the
upper integer boundary. No production implementation change was needed.

## Verification

The focused locked/offline test passed 1/1, and the full
`preview::registry::tests` module passed 3/3. Full command output is recorded in
`target/headstart/evidence/m1313-preview-registry-i64-max-20261024.log`.

```sh
./scripts/cargo-headstart test -p rustui --lib preview::registry::tests::impossible_i64_minimum_leaves_registry_available_for_maximum_boundary --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::registry::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --quiet
```

The local `cc-1.8.0` vendor package omitted four files. Cargo used a temporary
symlink overlay whose replacement package came from worktree 6390 after all 36
files listed in its `.cargo-checksum.json` matched. The overlay symlinks were
removed after verification; no vendor files were changed and `target/headstart`
was preserved.

This is local registry and host-mount evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
