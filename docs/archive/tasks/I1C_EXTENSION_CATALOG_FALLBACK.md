# I1C extension-aware catalog fallback

`Locale` accepts BCP 47 extension and private-use subtags, but catalog lookup
previously moved from an extension-bearing exact locale directly to its primary
language. For example, `en-GB-u-ca-gregory` skipped a registered `en-GB`
catalog and could resolve a less specific `en` translation. Lookup now tries
the extension-free locale before the primary language for both the requested
locale and configured fallback locale.

The locked/offline `rustui-i18n` package suite passed 19/19 unit tests and 1/1
catalog contract integration test. Evidence:
`target/headstart/evidence/localization-extension-fallback-tests-20261021.log`.
Rustfmt and scoped whitespace checks passed. This source-level check does not
cover native locale switching, visual output, or screen-reader behavior; those
acceptance gates remain open.

## M431 — extension-aware catalog fallback

Resolution tries the extension-free requested locale before its primary
language, then the extension-free configured fallback before its primary
language. Tests cover extension-bearing requested and fallback tags. The
locked/offline `rustui-i18n` package suite passed 19/19 unit tests and 1/1
catalog-contract integration test; rustfmt and scoped diff checks passed.
Native locale switching, visual output, and screen-reader acceptance remain
open. Evidence:
`target/headstart/evidence/localization-extension-fallback-tests-20261021.log`.
