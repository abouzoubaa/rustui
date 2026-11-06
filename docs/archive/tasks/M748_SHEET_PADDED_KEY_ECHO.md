# M748 — Sheet padded untranslated-key fallback

The Stage 8 Sheet accessible-name resolver treats a localization result that
echoes its message key as unresolved, then continues through the explicit name,
visible title, trigger, and identity fallback order. Before M748, Sheet compared
the raw result with the key, unlike the adjacent Dialog helper, so a result
containing the key plus whitespace was incorrectly accepted as a translation.

Sheet now compares trimmed values while retaining the existing nonblank check.
The focused regression supplies a whitespace-padded untranslated key and
verifies that the visible title is selected.

Verification: `./scripts/cargo-headstart test -p rustui --lib --locked --offline
components::overlay::sheet::tests -- --format terse` passed 19/19. Rust 1.97.1
`rustfmt --check --edition 2024 crates/rustui/src/components/overlay/sheet.rs`
and `git diff --check -- crates/rustui/src/components/overlay/sheet.rs
docs/tasks/M748_SHEET_PADDED_KEY_ECHO.md` passed.
