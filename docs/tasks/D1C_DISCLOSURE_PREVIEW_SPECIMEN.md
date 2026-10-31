# D1c Disclosure preview specimen

The canonical compiled-preview gallery includes an opt-in controlled RustUI
`Disclosure` specimen. Its descriptor stores a bounded nonblank label and an
`open` Boolean. The `toggle` event has an empty payload and maps to the
allowlisted `toggle` action; the controlled binding dispatches that signal and
then applies generation/revision-stamped Boolean feedback. Disclosure content
is compiled locally and is never serialized.

The focused regression covers schema bounds, the signal-only action mapping,
accepted open feedback, stale callback rejection, descriptor round-trip, and
rejection of a whitespace-only label without replacing the mounted preview.
Native activation, keyboard/focus, accessibility, localization, and
cross-platform acceptance remain separate gates.

## Verification

The focused locked/offline Playground regression passed 1/1, and the locked/offline
all-targets Playground check passed. Scoped rustfmt and diff checks passed.
Evidence: `target/headstart/evidence/m286-disclosure-specimen-test-20261020.log`
and `target/headstart/evidence/m286-disclosure-specimen-check-20261020.log`.
