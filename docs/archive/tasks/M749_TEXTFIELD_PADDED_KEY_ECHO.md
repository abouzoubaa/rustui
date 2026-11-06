# M749 — Stage 7 / detect padded TextField localization key echoes

TextField accessible-name resolution now compares trimmed catalog text and
message key when detecting an untranslated key echo. Blank and padded key-echo
results use the existing placeholder/explicit-name fallback, while genuine
translations are returned verbatim.

The focused regression `padded_localization_key_echo_uses_text_field_fallback`
passed 1/1 with `./scripts/cargo-headstart test -p rustui
components::input::text_field::tests::padded_localization_key_echo_uses_text_field_fallback
--lib --locked --offline`. Rust formatting and scoped `git diff --check` passed.
This is source-level validation; native accessibility and locale-switching
acceptance remain open. Test output:
`target/headstart/evidence/m749-textfield-padded-key-echo-test.log`.
