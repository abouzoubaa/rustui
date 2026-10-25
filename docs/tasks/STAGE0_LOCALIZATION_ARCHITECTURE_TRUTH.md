# Stage 0 — correct localization capability descriptions

The architecture and current file registry described plural expressions and
locale-sensitive number formatting as unsupported. Current source implements
bounded `plural`, `selectordinal`, and `select` expressions in
`crates/rustui-i18n/src/formatting.rs`, locale-aware integer and decimal
formatting, and catalog validation for argument and expression parity in
`crates/rustui-i18n/src/validation.rs`. The README summary also understated these
implemented capabilities.

Updated only the architecture overview, current file-registry entries, and
README capability summary. The supported formatter remains bounded; full CLDR,
date formatting, Fluent terms, and broader locale behavior are still incomplete.
This documentation-only correction was checked against current source; no
runtime code or acceptance status changed.
