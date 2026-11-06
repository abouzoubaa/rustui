# M771 — Stage 6 / place directional marks with localized number signs

Number formatting now chooses the directional mark from the active numbering
system, including `nu` overrides, and applies it to positive and negative
signs for integer and decimal output. Arabic digits use U+061C; Latin digits in
Arabic locales use U+200E. This follows the observed `Intl.NumberFormat`
behavior for the covered locale/numbering-system combinations.

The focused locked/offline regression passed (1/1), covering Arabic digits,
Arabic locales with Latin digits, and English with Arabic digits. Scoped
formatting and `git diff --check` passed. Broader locale/CLDR and native
locale-switching acceptance remain open.
