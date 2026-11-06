# M296 — Stage 9 / D1c Sheet preview specimen

## Status

The gallery implementation, focused regression, and locked/offline validation
are complete. Native Sheet acceptance gates remain open.

## Scope

The compiled-preview gallery includes an opt-in, signal-free Sheet built with
RustUI's public `Sheet` API. Its descriptor stores three bounded, nonblank
strings: `trigger_label` (`Open details`, 1–80 bytes), `title` (`Details`,
1–120 bytes), and `body_text` (`Preview sheet content`, 1–240 bytes). The
compiled factory supplies a generation-specific identity and constructs the
body with public `Text(body_text)`. Descriptors contain no view, content
closure, event, or action; the native Sheet trigger opens the sheet.

The focused regression covers schema defaults and bounds, the empty
event/action contract, descriptor round-trip, and rejection of blank imported
trigger, title, or body text without replacing the mounted preview.

Native Sheet appearance, trigger activation and dismissal, keyboard/focus,
accessibility-tree role/name and VoiceOver, localization, and cross-platform
behavior remain open acceptance gates.

## Verification

- Focused regression passed (1 passed, 0 failed):
  `cargo-headstart test --manifest-path /Users/Abium/Developer/rustui/Cargo.toml -p rustui-playground --lib sheet_specimen_round_trips_bounded_strings_and_rejects_blank_imports --locked --offline`.
- Playground all-targets check passed:
  `cargo-headstart check --manifest-path /Users/Abium/Developer/rustui/Cargo.toml -p rustui-playground --all-targets --locked --offline`.
- Both Cargo runs used the isolated target
  `target/headstart/candidates/m296-validation/target`.
- Full logs: `target/headstart/evidence/m296-focused-test.log` and
  `target/headstart/evidence/m296-playground-all-targets-check.log`.
- `rustfmt --edition 2024 --check` and scoped `git diff --check` passed.
- No native GUI acceptance was run; the canonical Playground was not launched
  or shut down.
