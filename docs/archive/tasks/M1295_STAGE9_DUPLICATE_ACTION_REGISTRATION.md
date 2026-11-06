# M1295 — Stage 9: preserve an allowlisted action after duplicate registration

## Contract

`ComponentCatalog::register_action` rejects an action reference that is already
registered. A failed duplicate registration must leave the existing allowlist
entry usable for preview validation.

## Change

Added a focused regression using the existing catalog and preview fixtures. It
attempts to register the existing `host.preview.report` action again, confirms
registration fails, and validates the original preview successfully. No
production implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::schema::tests::duplicate_action_registration_preserves_existing_allowlist_entry --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1295"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` package was incomplete. Cargo used a
temporary overlay combining the existing vendor entries with the package from
worktree 6390 after all 36 files listed in its `.cargo-checksum.json` matched.
The overlay was removed after the test; no vendor files were changed and the
target cache was not cleaned.

This is local action-allowlist validation evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
