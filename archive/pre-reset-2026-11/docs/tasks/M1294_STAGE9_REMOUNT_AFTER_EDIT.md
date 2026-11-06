# M1294 — Stage 9: discard edited state on successful remount

## Contract

A successful `PreviewHost::mount` creates a fresh generation at revision 0 from
the validated descriptor. Remounting the same descriptor after an edit must
replace the previous edited property state with the schema-resolved defaults;
the old stamp must no longer authorize host operations.

## Change

Added a focused host regression that edits the text property, successfully
remounts the same descriptor, and confirms the generation advances while the
revision resets to 0. The remounted snapshot contains the schema default
`hello`, and an edit using the pre-remount stamp is rejected. No production
implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::successful_remount_discards_edited_preview_state --locked --offline --config 'source.vendored-sources.directory="<temporary verified vendor overlay>"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` package was incomplete. Cargo used a
temporary overlay combining the existing vendor entries with the package from
worktree 6390 after all 36 files listed in its `.cargo-checksum.json` matched.
The overlay was removed after the test; no vendor files were changed and the
target cache was not cleaned.

This is source-level remount lifecycle evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
