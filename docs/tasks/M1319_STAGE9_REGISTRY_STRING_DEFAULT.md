# M1319 — Stage 9: validate string defaults against allowed values

## Contract

A `String` property default must be present in its declared `allowed_values`.
If it is absent, `PreviewRegistry::register` rejects the component without
occupying its ID; a corrected default under the same ID remains registerable
and mountable.

## Change

Added a registry regression that rejects default `stop` when the allowed value
is `run`, checks that the registry remains empty, then registers the same
component with default `run` and verifies the mounted value and render path.
No production implementation change was needed.

## Verification

The focused locked/offline test passed 1/1, and the complete
`preview::registry::tests` module passed 7/7. Full command output is recorded
in:

- `target/headstart/evidence/m1319-preview-registry-string-allowed-default-20261025.log`
- `target/headstart/evidence/m1319-preview-registry-tests-20261025.log`

The temporary vendor overlay used the checksum-verified replacement package
for the four absent local `cc-1.8.0` files. Its symlinks were removed
afterward; no vendor files were changed and `target/headstart` was preserved.

This is local registry and host-mount evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
