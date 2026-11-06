# M1296 — Stage 9: advance the revision on typed binding feedback

## Contract

`PreviewHost::binding_feedback` accepts owner-provided typed values only for
the exact current preview stamp and applies them through the preview schema.
An accepted changed value advances the revision, making later feedback with the
old stamp stale.

## Change

Added a focused host regression that feeds a string value through
`binding_feedback`, confirms the same generation advances to the next revision
and the resolved property updates, then rejects a late feedback value using
the original stamp while preserving the accepted value. No production
implementation change was needed.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::binding_feedback_updates_the_current_revision --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1295"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` package was incomplete. Cargo used a
temporary overlay combining the existing vendor entries with the package from
worktree 6390 after all 36 files listed in its `.cargo-checksum.json` matched.
The overlay was removed after the test; no vendor files were changed and the
target cache was not cleaned.

This records local typed host feedback behavior only. It does not resolve or
claim native text editing, selection, composition, or IME ownership. D1d
composition and D1b IME ownership decisions remain gated; broader native and
preview acceptance remain separate.
