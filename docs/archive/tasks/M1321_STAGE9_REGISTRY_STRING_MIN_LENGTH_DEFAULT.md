# M1321 — Stage 9: validate minimum string length defaults

## Contract

A String default must satisfy `min_length`, measured in Unicode characters.
Registration rejects a shorter default without occupying the component ID; a
corrected default under the same ID remains registerable and mountable.

## Change

Added a registry regression with minimum length two: default `é` is rejected,
then the same component ID registers with default `éé`. The test checks the
mounted value and render path. No production implementation change was needed.

## Verification

The focused locked/offline test passed 1/1, and the complete
`preview::registry::tests` module passed 9/9. Full command output is recorded
in:

- `target/headstart/evidence/m1321-preview-registry-string-min-length-default-20261025.log`
- `target/headstart/evidence/m1321-preview-registry-tests-20261025.log`

The temporary vendor overlay used the checksum-verified replacement package
for the four absent local `cc-1.8.0` files. Its symlinks were removed
afterward; no vendor files were changed and `target/headstart` was preserved.

This is local registry and host-mount evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
