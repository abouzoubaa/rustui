# M815: reject duplicate Unicode locale keys

## Scope

Reject ambiguous duplicate two-character keys inside a BCP 47 Unicode `u`
extension. In particular, repeated numbering-system key `nu` values must not
be accepted with first-value-wins behavior.

## Change

`Locale::parse` now detects duplicate Unicode extension keys case-insensitively
and rejects the locale tag. Valid single numbering-system overrides retain
their existing formatting behavior.

## Evidence

- `rejects_duplicate_unicode_extension_keys_case_insensitively` — 1 focused
  regression passed.
- `formats_integers_for_supported_locale_families` — 1 valid-override
  regression passed.

Full CLDR/ICU support and broader locale, RTL, accessibility, and
cross-platform acceptance remain open.
