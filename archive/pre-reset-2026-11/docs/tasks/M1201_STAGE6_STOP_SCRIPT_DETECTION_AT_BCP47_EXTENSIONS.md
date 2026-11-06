# M1201 — Stage 6 stop script detection at BCP 47 extensions

Explicit script detection now stops at BCP 47 singleton boundaries. A
four-letter value such as `Latn` or `Arab` inside a Unicode extension therefore
cannot override the direction implied by the locale's language.

The regression confirms `ar-u-ca-Latn` remains right-to-left and
`en-u-ca-Arab` remains left-to-right.

Verification: `./scripts/cargo-headstart test -p rustui-i18n
unicode_extension_values_do_not_override_locale_direction --locked --offline`
passed (1 passed, 0 failed; 75 filtered). Full CLDR/ICU/Fluent coverage and
native/browser RTL acceptance remain open.
