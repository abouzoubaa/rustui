# M1283 — Stage 9: failed preview mounts do not consume generations

## Contract

`PreviewHost::mount` validates a descriptor before changing mounted state or
generation. A rejected remount leaves the current stamp intact, and the next
valid remount advances generation exactly once.

## Change

Added a focused host regression that rejects a component-version mismatch,
checks the current stamp is unchanged, then confirms the next valid mount uses
the immediately following generation. The production implementation already
satisfied the contract.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::failed_mount_does_not_consume_a_generation --locked --offline --config 'source.vendored-sources.directory="/var/folders/t3/sszszn6s4xl0q_5x3rr0c_800000gn/T/rustui-m1282-vendor-io1nrqdl"' -- --exact
```

The reconstructed checkout lacks four checksum-listed files from its `cc`
vendor package. Cargo used the existing target and a temporary source overlay
combining the unchanged 91cb vendor entries with the checksum-verified
`cc-1.8.0` package from worktree 6390. The overlay was removed afterward; no
vendor files were changed and the target cache was not cleaned.

This is source-level lifecycle regression evidence; broader native preview
acceptance remains separate. D1d composition and D1b IME ownership decisions
remain gated.
