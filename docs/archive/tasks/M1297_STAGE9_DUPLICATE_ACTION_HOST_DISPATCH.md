# M1297 — Stage 9: preserve host dispatch after duplicate action registration

## Contract

A duplicate `PreviewRegistry::register_action` call returns an error and must
not revoke the action that was already registered. A preview using that action
must still mount, and its current signal must still resolve to the allowlisted
action through `PreviewHost::dispatch`.

## Change

Added a focused runtime host regression using the existing `count` action. It
rejects the duplicate registration through the registry held by the host,
mounts the preview, and confirms the current `activate` signal still dispatches
to `count`. No production implementation change was needed. This adds the host
integration path to M1295's catalog-level allowlist-preservation regression.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::duplicate_registry_action_registration_preserves_host_dispatch --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1295"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` package was incomplete. Cargo used a
temporary overlay combining the existing vendor entries with the package from
worktree 6390 after all 36 files listed in its `.cargo-checksum.json` matched.
The overlay was removed after the test; no vendor files were changed and the
target cache was not cleaned.

This is local registry-to-host action dispatch evidence; native and broader
preview acceptance remain separate. D1d composition and D1b IME ownership
decisions remain gated.
