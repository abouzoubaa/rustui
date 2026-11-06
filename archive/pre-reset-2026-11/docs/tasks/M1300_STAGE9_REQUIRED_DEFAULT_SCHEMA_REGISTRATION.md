# M1300 — Stage 9: keep the catalog reusable after a required/default conflict

## Contract

A property schema cannot be both required and defaulted. Registration validates
this constraint before inserting the component, so rejection must leave the
component ID available for a corrected schema.

## Change

Added a focused catalog regression that takes the existing defaulted `label`
property, marks it required, and confirms registration returns `Invalid`. It
then registers the corrected component schema under the same ID, registers the
preview action, and confirms the existing preview validates. No production
implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::schema::tests::required_default_conflict_does_not_poison_component_registration --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1295"' -- --exact --quiet
```

The reconstructed checkout's `cc-1.8.0` package was incomplete. Cargo used a
temporary overlay combining the existing vendor entries with the package from
worktree 6390 after all 36 files listed in its `.cargo-checksum.json` matched.
The overlay was removed after the test; no vendor files were changed and the
target cache was not cleaned.

This is local schema-registration and preview-validation evidence; native and
broader preview acceptance remain separate. D1d composition and D1b IME
ownership decisions remain gated.
