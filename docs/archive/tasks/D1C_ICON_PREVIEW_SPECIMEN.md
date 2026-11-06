# D1c Icon preview specimen

The canonical compiled-preview gallery includes an opt-in, signal-free Search
Icon specimen using the public RustUI `Icon` API. Its descriptor exposes one
editable `accessibility_label` property (default `Search`, 1–80 characters).
The factory uses the existing Playground Search asset, a fixed 24-point size,
and a generation-specific identity. Blank labels fall back to `Search` before
calling the public builder, which rejects blank accessible names.

The visible mount control is `Preview Icon specimen`. The existing preview
inspector and descriptor codec provide property editing and import/export. The
event map is empty; no payload or descriptor contract changed. The focused
locked/offline regression passed 1/1, and the isolated locked/offline
Playground all-targets check passed. Evidence:
`target/headstart/evidence/m272-icon-preview-tests-20261020.log` and
`target/headstart/evidence/m272-icon-playground-check-20261020.log`. Native icon
appearance, accessibility-tree output, keyboard/focus, localization, and
cross-platform acceptance remain open.

## M273 native appearance and AX name

The disposable Playground rendered the Search glyph and exposed a native AX
image named Search. Initial inspection also revealed that the string inspector
showed the previous Text preview value. M281 fixes the inspector binding to
initialize and read from the active descriptor. The rebuilt app showed
`accessibility_label = Search` and image name Search. Evidence:
`target/headstart/evidence/m273-icon-native-acceptance-20261020.json`.
Keyboard/focus, VoiceOver, localization, theme, RTL, and cross-platform
acceptance remain open; no screenshot file was retained.

## M724 — Stage 9 inspector icon-size control

The Icon specimen now exposes its previously fixed size through the preview
inspector, using the public `IconView::size` API. The value keeps the existing
24 default and is bounded to 8–64. The focused descriptor regression verifies
size 36 survives import/export; native visual acceptance remains open.
