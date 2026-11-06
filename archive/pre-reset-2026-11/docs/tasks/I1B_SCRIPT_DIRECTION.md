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
## M443 — N’Ko language fallback

The focused regression confirms that N’Ko language selection (`nqo`) resolves
to right-to-left when direction falls back to the language, and passed 1/1
(`target/headstart/evidence/nko-direction-m443-test.log`). This is unit evidence;
it does not establish native RTL layout acceptance.


## M561 Yezidi explicit-script direction
Added the explicit RTL script code `Yezi`; `ku-Yezi` resolves to RTL while `ku-Latn` remains LTR. Focused direction tests passed 4/4 in `target/headstart/evidence/m561-explicit-yezi-script-direction-20261021.log`. This adds one script code and does not claim comprehensive RTL-script coverage or native layout acceptance.

## M720 — Central Kurdish language fallback

Central Kurdish (`ckb`) now defaults to right-to-left when a locale omits its
script subtag. An explicit `Latn` script still selects left-to-right. This
extends the bounded language fallback list; it does not claim complete BCP 47
or RTL-language coverage. The focused locked/offline regression is recorded in
`target/headstart/evidence/m720-central-kurdish-direction-test.log`.
