# M746 — Stage 7 / reject blank Picker option names

Picker option strings are the visible and accessible labels for selectable
items. The constructor now rejects empty and whitespace-only options before
building retained Select state. Nonblank option strings remain unchanged.

The focused regression `picker::tests::rejects_blank_accessible_option_names`
passed 1/1 with `./scripts/cargo-headstart test -p rustui
picker::tests::rejects_blank_accessible_option_names --lib --locked --offline`.
Rust formatting and scoped `git diff --check` passed. This is source-level
validation; native option announcements, VoiceOver, keyboard/focus, RTL, and
cross-platform acceptance remain open. Test output:
`target/headstart/evidence/m746-picker-blank-option-test.log`.
