# I1B Explicit Script Direction

## Behavior

`LayoutDirection::for_locale` now checks an explicit BCP 47 script subtag
before using the primary language. Recognized RTL scripts select RTL; any
other explicit script, including `Latn`, selects LTR. If the locale has no
script subtag, the existing supported RTL-language list determines direction.

Examples: `ar` is RTL, `ar-Latn` is LTR, and `en-Arab` and `en-Hebr` are RTL.

## Scope and limits

This change does not alter `Locale::parse` or broaden its accepted syntax. It
recognizes a script only in the script position before a region or other later
subtags. Script coverage is limited to the RTL scripts listed in the direction
implementation; unsupported scripts default to LTR when explicitly present.

## Verification

Focused unit coverage checks language fallback, Latin override, Arab/Hebr
script selection, and that a later variant-like subtag does not act as a
script.
