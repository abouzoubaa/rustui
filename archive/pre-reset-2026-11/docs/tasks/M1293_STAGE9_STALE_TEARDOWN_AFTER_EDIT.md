# M1293 — Stage 9: reject stale teardown after preview edits

## Contract

`PreviewHost::teardown` accepts only the exact current preview stamp. After an
edit advances the revision, a teardown request carrying the earlier stamp must
be rejected and leave the current mounted preview intact.

## Change

Added a focused host regression that mounts a text preview, edits its text,
and attempts teardown with the original stamp. The test confirms the generation
is unchanged, the revision advances, stale teardown fails, and the current
stamp and edited value remain mounted. No production implementation change was
needed.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::stale_revision_cannot_teardown_edited_preview --locked --offline --config 'source.vendored-sources.directory="<temporary verified vendor overlay>"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` package was incomplete. Cargo used a
temporary overlay combining the existing vendor entries with the package from
worktree 6390 after all 36 files listed in its `.cargo-checksum.json` matched.
The overlay was removed after the test; no vendor files were changed and the
target cache was not cleaned.

This is source-level stale-stamp lifecycle evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
