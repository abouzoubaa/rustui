# M1289 — Stage 9: preserve no-op edits at the revision limit

## Contract

An unchanged property batch preserves its exact current stamp and does not
consume a revision. A no-op remains valid even when the current revision is
`u64::MAX`; revision exhaustion applies only when a changed edit needs to
advance the counter.

## Change

Added a host-level regression that first makes a default value explicit, sets
the current stamp to `u64::MAX`, then submits the same value again. The edit
succeeds with the unchanged stamp and preserves the complete snapshot. The
production implementation already met the contract.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::no_op_edit_at_revision_limit_preserves_the_stamp --locked --offline --config 'source.vendored-sources.directory="/var/folders/t3/sszszn6s4xl0q_5x3rr0c_800000gn/T/rustui-m1289-vendor-fe9t7a0n"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` vendor package is incomplete. Cargo
used a temporary source overlay combining the existing vendor entries with
the checksum-verified package from worktree 6390. The overlay was removed
afterward; no vendor files were changed and the target cache was not cleaned.

This is source-level preview regression evidence; broader native acceptance
remains separate. D1d composition and D1b IME ownership decisions remain
gated.
