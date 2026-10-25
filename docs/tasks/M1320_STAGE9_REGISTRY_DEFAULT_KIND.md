# M1320 — Stage 9: validate default values match property kinds

## Contract

A `PropertySchema` default must use the `PropertyValue` variant declared by
its `PropertyKind`. Registration rejects a mismatched default without
occupying the component ID; a corrected default under the same ID remains
registerable and mountable.

## Change

Added a registry regression that declares a String property with a Boolean
default, checks registration fails and leaves the catalog empty, then registers
the same component with the String default `ready` and verifies the mounted
value and render path. No production implementation change was needed.

## Verification

The focused locked/offline test passed 1/1, and the complete
`preview::registry::tests` module passed 8/8. Full command output is recorded
in:

- `target/headstart/evidence/m1320-preview-registry-type-default-20261025.log`
- `target/headstart/evidence/m1320-preview-registry-tests-20261025.log`

The temporary vendor overlay used a replacement `cc-1.8.0` package whose 36
manifest-listed files matched their checksums. Its symlinks were removed
afterward; no vendor files were changed and `target/headstart` was preserved.

This is local registry and host-mount evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
