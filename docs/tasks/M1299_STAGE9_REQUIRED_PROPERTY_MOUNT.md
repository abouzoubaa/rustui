# M1299 — Stage 9: preserve the active preview on a missing required property

## Contract

A property schema marked `required` without a default must receive a value in
the preview descriptor. `PreviewHost::mount` validates the descriptor before
replacing the active snapshot; a missing required property must leave that
snapshot intact.

## Change

Added a host regression using a required text property with no default. It
first mounts a descriptor that supplies the value, then attempts to mount one
that omits it. The second mount returns `MissingRequiredProperty` and preserves
the full current snapshot and stamp. No production implementation change was
needed.

## Verification

The focused locked/offline RustUI library test passed 1/1, and the complete
`preview::host::tests` module passed 18/18:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::missing_required_property_mount_preserves_current_preview --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1295"' -- --exact
./scripts/cargo-headstart test -p rustui --lib preview::host::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1295"' -- --quiet
```

The reconstructed checkout's `cc-1.8.0` package was incomplete. Cargo used a
temporary overlay combining the existing vendor entries with the package from
worktree 6390 after all 36 files listed in its `.cargo-checksum.json` matched.
The overlay was removed after the test; no vendor files were changed and the
target cache was not cleaned.

This is local descriptor-validation and mount-preservation evidence; native
and broader preview acceptance remain separate. D1d composition and D1b IME
ownership decisions remain gated.
