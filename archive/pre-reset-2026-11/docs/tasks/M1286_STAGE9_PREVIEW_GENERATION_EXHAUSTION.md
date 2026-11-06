# M1286 — Stage 9: preserve the mounted preview when generations are exhausted

## Contract

Mount validates before replacing the active preview and must advance the
generation without wrapping. If the generation counter is exhausted, mount
fails and leaves the current instance intact.

## Change

Added a host-level regression that mounts a preview, sets the internal
generation counter to `u64::MAX`, and attempts another valid mount. It checks
that the exhaustion error leaves the current snapshot unchanged and that its
allowlisted action still dispatches. The production implementation already
met the contract.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::generation_exhaustion_preserves_the_mounted_preview --locked --offline --config 'source.vendored-sources.directory="/var/folders/t3/sszszn6s4xl0q_5x3rr0c_800000gn/T/rustui-m1286-vendor-qlw5mi8q"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` vendor package is incomplete. Cargo
used a temporary source overlay combining the existing vendor entries with
the checksum-verified package from worktree 6390. The overlay was removed
afterward; no vendor files were changed and the target cache was not cleaned.

This is source-level preview regression evidence; broader native acceptance
remains separate. D1d composition and D1b IME ownership decisions remain
gated.
