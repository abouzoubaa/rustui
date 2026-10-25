# M1322 — Stage 9: validate Unicode maximum-length defaults

## Contract

`String` defaults must satisfy `max_length`, measured in Unicode characters.
An overlong default must be rejected without occupying the component ID; a
corrected default under the same ID must remain registerable and mountable.

## Change

Added a registry regression with `max_length` two. Default `ééé` is rejected,
leaving the registry empty. The same component ID then registers with `éé`
(two Unicode characters encoded as four UTF-8 bytes); mounting resolves the
corrected default and rendering succeeds. No production validation change was
needed.

## Verification

The focused locked/offline test passed 1/1, and the complete
`preview::registry::tests` module passed 10/10. Full command output is recorded
in:

- `target/headstart/evidence/M1322-preview-registry-unicode-max-length-default.log`
- `target/headstart/evidence/M1322-preview-registry-tests.log`

The temporary vendor overlay used the replacement `cc-1.8.0` package whose 36
manifest-listed files matched their checksums. Its symlinks were removed
afterward; no vendor files were changed and `target/headstart` was preserved.

This is local registry and host-mount evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
