# M1318 — Stage 9: validate string defaults during registry registration

## Contract

A `String` property default must satisfy its declared `max_length`. If a
default is too long, `PreviewRegistry::register` rejects the component without
occupying its ID; a corrected default under the same ID remains registerable
and mountable.

## Change

Added a registry regression that rejects the default `long` with `max_length`
3, checks the registry remains empty, then registers the same component with
default `run` and verifies the mounted value and render path. No production
implementation change was needed.

## Verification

The focused locked/offline test passed 1/1, and the complete
`preview::registry::tests` module passed 6/6. Full command output is recorded
in:

- `target/headstart/evidence/m1318-preview-registry-string-default-20261025.log`
- `target/headstart/evidence/m1318-preview-registry-tests-20261025.log`

The temporary vendor overlay replaced the four absent local `cc-1.8.0` files
with a checksum-verified package from worktree 6390. All overlay symlinks were
removed afterward; no vendor files were changed and `target/headstart` was
preserved.

This is local registry and host-mount evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
