# M750 — Stage 7 / detect padded ComboBox localization key echoes

ComboBox accessible-name resolution now compares trimmed message text and key
when detecting an untranslated key echo. Blank and padded key-echo results use
the constructor-label fallback; a real translation is still returned verbatim.
This matches the existing Dialog and Sheet accessible-name resolution policy.

The focused regression `padded_localization_key_echo_uses_constructor_label_fallback`
passed 1/1 with `./scripts/cargo-headstart test -p rustui
components::input::combo_box::tests::padded_localization_key_echo_uses_constructor_label_fallback
--lib --locked --offline`. Rust formatting and scoped `git diff --check` passed.
Native accessibility and locale-switching acceptance remain open. Test output:
`target/headstart/evidence/m750-combobox-padded-key-echo-test.log`.
