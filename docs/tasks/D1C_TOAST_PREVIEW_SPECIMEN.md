# M295 — Stage 9 / D1c Toast preview specimen

## Status

Source implementation, focused regression, and locked/offline validation are
complete. Native notification and cross-platform acceptance gates remain open.

## Scope

The compiled-preview gallery exposes an opt-in Toast specimen through RustUI's
public `Toast` API. Its descriptor stores a bounded, nonblank `message` (default
`Changes saved`, 1–120 bytes) and an optional bounded, nonblank `title` (1–120
bytes when present). The compiled factory fixes `ToastKind::Info`, automatic
dismissal, and action behavior; no notification ID, closure, or executable
behavior is serialized.

The specimen exposes one empty-payload `show` event, mapped to the fixed
allowlisted `toast_show` action, and a visible `Preview Toast specimen` mount
control. The focused regression covers defaults and bounds, the event/action
contract, descriptor round-trip, and blank imported message/title rejection
without replacing the mounted preview.

Native notification appearance and delivery, automatic dismissal, accessibility
tree and VoiceOver behavior, keyboard/focus, localization, and cross-platform
acceptance remain open. A source test or successful build does not close these
gates.

## Verification

- Focused regression passed (1 passed, 0 failed):
  `cargo-headstart test --manifest-path /Users/Abium/Developer/rustui/Cargo.toml -p rustui-playground --lib toast_specimen_round_trips_bounded_message_and_optional_title --locked --offline`.
- Playground all-targets check passed:
  `cargo-headstart check --manifest-path /Users/Abium/Developer/rustui/Cargo.toml -p rustui-playground --all-targets --locked --offline`.
- Both Cargo runs used the isolated target
  `target/headstart/candidates/m295-validation/target`.
- Full logs: `target/headstart/evidence/m295-focused-test.log` and
  `target/headstart/evidence/m295-playground-all-targets-check.log`.
- `rustfmt --edition 2024 --check` and scoped `git diff --check` passed.
- No native GUI acceptance was run; the canonical Playground was not launched
  or shut down.
