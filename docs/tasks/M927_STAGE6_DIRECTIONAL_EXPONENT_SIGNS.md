# M927 — Stage 6 / keep localized exponent signs directionally stable

The bounded decimal formatter localized scientific exponent digits but left a
signed exponent's ASCII `+` or `-` without the directional mark used for other
localized number signs. It now places the active locale/numbering-system mark
before signed exponent signs, preserving the exponent marker, sign, and digits.

The focused regression covers Arabic digits, Arabic with `nu-latn`, and English
with `nu-arab`. The locked/offline focused test passed 1/1; `rustfmt --check`
and scoped `git diff --check` passed. Full CLDR/ICU support and native RTL
acceptance remain outside this slice.
