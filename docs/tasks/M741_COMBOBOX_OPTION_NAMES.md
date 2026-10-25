# M741 — Stage 7 / reject blank ComboBox option names

ComboBox option strings are also the visible and accessible option labels. The
constructor now rejects empty or whitespace-only options before constructing
the retained Select state, preventing selectable popup entries without names.
Nonblank option strings and their whitespace are preserved verbatim.

The focused regression `rejects_blank_accessible_option_names` passed 1/1 with
`./scripts/cargo-headstart test -p rustui rejects_blank_accessible_option_names
--lib --locked --offline`. Rust formatting and scoped `git diff --check` passed.
This is source-level validation; native popup, VoiceOver, keyboard/focus, RTL,
and cross-platform acceptance remain open. Test output:
`target/headstart/evidence/m741-combobox-blank-option-test.log`.
