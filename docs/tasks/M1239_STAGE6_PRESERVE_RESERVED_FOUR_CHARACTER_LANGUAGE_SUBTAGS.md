# M1239 — Preserve RFC 5646 four-letter primary language subtags

An attempted M1231 parser restriction was incorrect: RFC 5646 §2.1 explicitly
allows `4ALPHA` primary language subtags as reserved future use. The rejection
was reverted. A positive regression confirms `abcd` and `ABCD-Latn` parse, with
the latter normalized to `abcd-Latn`.

```sh
./scripts/cargo-headstart test -p rustui-i18n \
  accepts_four_character_primary_language_subtags
```

Result: passed (1/1). Source: [RFC 5646 §2.1](https://www.rfc-editor.org/rfc/rfc5646.txt),
grammar lines 238–243. The Stage 6 locale strategy, broader locale data,
RTL, and native/browser accessibility acceptance remain open.
