# M290 — D1c Image preview specimen

The compiled-preview gallery adds an opt-in `Image` specimen using the checked-in
`assets/playground-sample.png` Playground asset. The fixed filesystem path is
constructed by the compiled factory and is not part of the descriptor. The
schema exposes only a bounded, nonblank accessibility label and a `fit` string
restricted to RustUI's public `ImageFit` choices.

The focused regression covers schema defaults and bounds, the constrained fit
values, absence of serialized source paths or actions, descriptor round-trip,
and rejection of a blank imported accessibility label without replacing the
mounted preview.

Image decoding/rendering, VoiceOver behavior, keyboard/focus, localization, and
cross-platform acceptance remain separate gates.

## Verification

The focused locked/offline Image regression passed 1/1. The locked/offline
Playground `--all-targets` check passed. Both used the isolated
`target/headstart/candidates/m290-validation/target` Headstart target. Scoped
rustfmt and `git diff --check` passed. Evidence:
`target/headstart/evidence/m290-image-preview-test-20261020.log` and
`target/headstart/evidence/m290-image-preview-check-20261020.log`.

These source checks do not establish image decoding/rendering, VoiceOver,
localization, or cross-platform acceptance.
