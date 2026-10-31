# M751 — DatePicker padded untranslated-key fallback

The DatePicker accessible-name contract uses the explicit or visible label as
fallback when localization returns a blank string or the untranslated message
key. The resolver previously compared the raw localized string to the key, so a
key echoed with surrounding whitespace was treated as a translation.

The resolver now compares trimmed values while preserving nonblank translated
text. A focused regression verifies that a padded key echo resolves to the
existing fallback.

Verification: `./scripts/cargo-headstart test -p rustui --lib --locked --offline
components::input::date_picker::tests -- --format terse` passed 13/13. Rust 1.97.1
`rustfmt --check --edition 2024 crates/rustui/src/components/input/date_picker.rs`
and `git diff --check -- crates/rustui/src/components/input/date_picker.rs
docs/tasks/M751_DATEPICKER_PADDED_KEY_ECHO.md` passed.
