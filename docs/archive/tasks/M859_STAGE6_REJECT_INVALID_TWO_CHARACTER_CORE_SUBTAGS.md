# M859 — Stage 6 / reject invalid two-character core subtags

`Locale::parse` now rejects unrecognized two-character subtags in the core
language tag, including invalid numeric and alphanumeric forms such as
`en-12` and `en-a1`. The same two-character values remain accepted where the
Unicode extension or private-use grammar permits them (`en-u-12` and
`en-x-12`). A focused regression passed 1/1; scoped formatting and diff checks
passed. See `crates/rustui-i18n/src/locale.rs`.

This is a bounded parser correction. Full locale-data coverage, VoiceOver,
RTL, and cross-platform acceptance remain open.
