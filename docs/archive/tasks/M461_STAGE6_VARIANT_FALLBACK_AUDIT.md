# M461 Stage 6 locale fallback audit

`Catalogs::resolve` tried the exact locale, its extension-free form, and the
primary language. That skipped intermediate variants: for example,
`sl-rozaj-biske-u-nu-latn` fell through to `sl` even when an `sl-rozaj`
catalog was registered. The resolver now removes extensions, then truncates
one trailing subtag at a time through the primary language, for both requested
and configured fallback locales.

The regression test registers `sl` and `sl-rozaj`, requests
`sl-rozaj-biske-u-nu-latn`, and asserts the `sl-rozaj` message wins. The audit
also reviewed locale parsing and `LayoutDirection::for_locale`; no additional
bounded direction or extension-boundary defect was established.

## Validation

- Locked/offline `rustui-i18n` library tests: 21 passed.
- Rustfmt check and scoped `git diff --check`: passed.
- Test log: `target/headstart/evidence/stage6-variant-fallback-audit-20261021.log`.
- Native locale switching and RTL GUI acceptance were not run.
