# M1290 — Stage 9: preserve a compiled factory after invalid duplicate registration

## Contract

`PreviewRegistry::register` rejects event schemas with payload properties and
keeps the existing catalog schema and compiled factory intact when that
rejection occurs for an already registered component ID.

## Change

Added a runtime regression that registers a component, attempts to register a
same-ID replacement with a payload-bearing event, and confirms rejection
preserves the original schema. Mounting and rendering then invokes the original
factory only. No production implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::registry::tests::payload_event_duplicate_keeps_the_original_schema_and_factory --locked --offline --config 'source.vendored-sources.directory="/var/folders/t3/sszszn6s4xl0q_5x3rr0c_800000gn/T/rustui-m1290-vendor-iccff3p8"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` vendor package is incomplete. Cargo
used a temporary source overlay combining the existing vendor entries with
the checksum-verified package from worktree 6390. The overlay was removed
afterward; no vendor files were changed and the target cache was not cleaned.

This is source-level registry evidence; broader native preview acceptance
remains separate. D1d composition and D1b IME ownership decisions remain
gated.
