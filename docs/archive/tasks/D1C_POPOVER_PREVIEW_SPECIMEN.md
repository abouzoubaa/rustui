# D1c Popover preview specimen

The canonical compiled-preview gallery exposes an opt-in public RustUI
`Popover` specimen. Its descriptor stores a bounded nonblank trigger label and
controlled `open` Boolean; the content remains a compiled RustUI view. It
declares no events, callbacks, or serialized views. Open and close feedback use
the gallery's generation/revision-stamped Boolean binding path.

The focused regression covers the schema, empty event set, accepted open/close
feedback, descriptor round-trip, and rejection of a whitespace-only trigger
without replacing the mounted preview. Native open/dismiss, keyboard/focus,
accessibility, localization, and cross-platform acceptance remain separate
gates.

## Verification

The focused locked/offline Playground library test passed 1/1 in an isolated
target directory. `rustfmt --check` and the scoped `git diff --check` passed.
Evidence: `target/headstart/evidence/d1c-popover-specimen-test-20261020.log`.
