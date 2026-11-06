# M1287 — Stage 9: preserve the mounted preview when revisions are exhausted

## Contract

Edits apply typed property batches atomically to the exact current stamp. A
changed edit advances the revision without wrapping; revision exhaustion
rejects the edit without changing the mounted preview.

## Change

Added a host-level regression that mounts a preview, sets its current revision
to `u64::MAX`, and submits a valid changed property value. It checks that the
exhaustion error preserves the snapshot and that the current allowlisted action
still dispatches. The production implementation already met the contract.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::revision_exhaustion_preserves_the_mounted_preview --locked --offline --config 'source.vendored-sources.directory="/var/folders/t3/sszszn6s4xl0q_5x3rr0c_800000gn/T/rustui-m1287-vendor-z534uf63"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` vendor package is incomplete. Cargo
used a temporary source overlay combining the existing vendor entries with
the checksum-verified package from worktree 6390. The overlay was removed
afterward; no vendor files were changed and the target cache was not cleaned.

This is source-level preview regression evidence; broader native acceptance
remains separate. D1d composition and D1b IME ownership decisions remain
gated.
