# M1288 — Stage 9: keep preview registration reusable after schema rejection

## Contract

`PreviewRegistry::register` accepts only signal-only component schemas and
adds a valid schema and compiled factory atomically. Rejecting an invalid
schema must leave the registry able to accept a corrected schema with the same
component ID.

## Change

Added a runtime regression that rejects a payload-bearing event schema,
confirms the catalog remains empty, then registers the corrected schema under
the same ID, mounts its preview, and renders through the accepted factory. No
production implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::registry_can_retry_after_rejecting_a_payload_event_schema --locked --offline --config 'source.vendored-sources.directory="/var/folders/t3/sszszn6s4xl0q_5x3rr0c_800000gn/T/rustui-m1288-vendor-9pp85zj3"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` vendor package is incomplete. Cargo
used a temporary source overlay combining the existing vendor entries with
the checksum-verified package from worktree 6390. The overlay was removed
afterward; no vendor files were changed and the target cache was not cleaned.

This is source-level registry evidence; broader native preview acceptance
remains separate. D1d composition and D1b IME ownership decisions remain
gated.
