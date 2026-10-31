# M1291 — Stage 9: refresh render factories after preview edits

## Contract

`render_factory()` captures an immutable preview snapshot. A previously
captured factory remains bound to its original revision, while a newly captured
factory reflects the current preview after an accepted edit.

## Change

Added an integration regression that captures a factory at revision 0, edits a
string property, and confirms the old factory still renders the old value. A
new factory then renders the edited value at revision 1. No production
implementation change was needed.

## Verification

The focused locked/offline RustUI integration test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --test preview_host_render_lifecycle captured_factory_keeps_its_revision_while_a_new_factory_sees_an_edit --locked --offline --config 'source.vendored-sources.directory="/var/folders/t3/sszszn6s4xl0q_5x3rr0c_800000gn/T/rustui-m1291-vendor-u5d_gvbg"' -- --exact
```

The reconstructed checkout's `cc-1.8.0` vendor package is incomplete. Cargo
used a temporary source overlay combining the existing vendor entries with
the checksum-verified package from worktree 6390. The overlay was removed
afterward; no vendor files were changed and the target cache was not cleaned.

This is source-level render-factory evidence; native edit-state and broader
preview acceptance remain separate. D1d composition and D1b IME ownership
decisions remain gated.
