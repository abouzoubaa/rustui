# M1282 — Stage 9: preserve the factory after duplicate registration

## Contract

`PreviewRegistry::register` adds a catalog schema and its compiled factory
atomically. A duplicate component ID must be rejected without replacing the
factory already registered for that ID.

## Change

Added a runtime regression that registers one factory, rejects a second
registration for the same component ID, then mounts and renders the component.
The original factory runs and the catalog still contains one component. The
production implementation required no change.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::registry::tests::duplicate_registration_keeps_the_original_factory --locked --offline --config 'source.vendored-sources.directory="/var/folders/t3/sszszn6s4xl0q_5x3rr0c_800000gn/T/rustui-m1282-vendor-io1nrqdl"' -- --exact
```

The reconstructed checkout lacks four checksum-listed files from the `cc`
vendor package. The check used the existing persistent target and a temporary
Cargo source overlay that referenced the unchanged 91cb vendor tree plus the
checksum-verified `cc-1.8.0` package in worktree 6390. The temporary overlay
was removed afterward. No vendor files were changed, and the existing target
cache was retained without cleaning.

This is source-level registry evidence; native and broader preview acceptance
remain separate. D1d composition and D1b IME ownership decisions remain gated.
