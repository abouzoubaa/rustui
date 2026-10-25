# M1284 — Stage 9: reject duplicate property updates atomically

## Contract

`PreviewHost::edit` applies typed property batches atomically. A batch that
contains the same property ID more than once is rejected without changing the
mounted preview or its revision.

## Change

Added a host-level regression that submits two different values for the same
property, expects rejection, and confirms both the mounted stamp and resolved
property value are unchanged. The production implementation already met the
contract.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::duplicate_property_update_batch_keeps_current_preview_unchanged --locked --offline --config 'source.vendored-sources.directory="/var/folders/t3/sszszn6s4xl0q_5x3rr0c_800000gn/T/rustui-m1282-vendor-io1nrqdl"' -- --exact
```

The reconstructed checkout lacks four checksum-listed files from its `cc`
vendor package. Cargo used the existing target and a temporary source overlay
combining the unchanged 91cb vendor entries with the checksum-verified
`cc-1.8.0` package from worktree 6390. The overlay was removed afterward; no
vendor files were changed and the target cache was not cleaned.

This is source-level preview regression evidence; broader native acceptance
remains separate. D1d composition and D1b IME ownership decisions remain
gated.
