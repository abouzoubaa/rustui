# M1285 — Stage 9: reject signals from a prior preview generation

## Contract

`PreviewSignal` carries its preview stamp, and `PreviewHost::dispatch` accepts
only signals for the exact current stamp. Reusing a preview ID on remount must
advance generation so signals captured from the prior mount are stale.

## Change

Added a host-level regression that captures an allowlisted signal, remounts the
same preview, and confirms that dispatch rejects the old signal while accepting
a signal captured from the current generation. The production implementation
already met the contract.

## Verification

The focused locked/offline RustUI library test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::host::tests::remount_rejects_signals_from_the_previous_generation --locked --offline --config 'source.vendored-sources.directory="/var/folders/t3/sszszn6s4xl0q_5x3rr0c_800000gn/T/rustui-m1285-vendor-sijyfffp"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` vendor package is incomplete. Cargo
used a temporary source overlay combining the existing vendor entries with
the checksum-verified package from worktree 6390. The overlay was removed
afterward; no vendor files were changed and the target cache was not cleaned.

This is source-level preview regression evidence; broader native acceptance
remains separate. D1d composition and D1b IME ownership decisions remain
gated.
