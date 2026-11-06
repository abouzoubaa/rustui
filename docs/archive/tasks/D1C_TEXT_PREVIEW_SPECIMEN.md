# M293 — Stage 9 / D1c Text preview specimen

## Scope

The compiled-preview gallery has an opt-in, signal-free `Text` specimen built with RustUI's public `Text` API. Its descriptor stores only a bounded, nonblank `value` string. The compiled factory constructs the view; descriptors contain no view, closure, style record, event, or action.

The focused regression covers the schema default and bound, empty event/action contracts, descriptor round-trip, and rejection of a blank imported value without replacing the mounted preview.

Native text appearance, wrapping, accessibility-tree role, keyboard/focus, VoiceOver, localization, and cross-platform behavior remain separate acceptance gates.

## Verification

- Focused locked/offline regression passed: `text_specimen_round_trips_bounded_value_without_events_or_actions` (1 passed). Full log: `target/headstart/evidence/m293-focused-test.log`.
- Locked/offline Playground all-targets check passed. Full log: `target/headstart/evidence/m293-playground-all-targets-check.log`.
- `rustfmt --edition 2024 --check apps/rustui-playground/src/stories/compiled_preview_gallery.rs` passed.
- Scoped `git diff --check` for the implementation and this task record passed.
- Cargo validation used isolated target `target/headstart/candidates/m293-validation/target`.
- No native GUI acceptance was run; the canonical Playground was not launched or shut down.
## M442 — explicit Regular text weight

The public `Text::font_weight(TextWeight::Regular)` choice resolves to the
explicit normal weight even when the theme supplies weight 510; leaving the
weight unspecified still uses that theme value. The focused unit test passed
1/1 (`target/headstart/evidence/text-regular-weight-m442-test.log`). This
checks weight resolution, not native font rendering or visual appearance.
