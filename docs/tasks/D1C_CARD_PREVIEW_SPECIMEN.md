# M359 — Stage 9 / D1c Card preview specimen

## Scope

The canonical compiled-preview gallery registers an opt-in `card-specimen`
using the public RustUI `Card` builder. Its schema exposes a bounded `label`
string and a `border` Boolean. The factory supplies the card ID locally and
uses compiled `Text` content; descriptors cannot supply views, closures,
callbacks, or executable behavior. The event schema and mounted action map are
empty.

## Validation

- The focused gallery test verifies the schema, empty event/action maps, and
  descriptor import/export for the label and border values.
- The gallery test module passed 51/51 tests through
  `./scripts/cargo-headstart test -p rustui-playground --lib
  stories::compiled_preview_gallery::tests --locked --offline`.
- The locked/offline Playground all-targets check passed.
- No GUI acceptance ran. Native appearance, theme, accessibility, and
  cross-platform checks remain open.

Evidence: `target/headstart/evidence/d1c-layout-specimens-m359-tests-20261021.log`
and `target/headstart/evidence/d1c-layout-specimens-m359-check-20261021.log`.
