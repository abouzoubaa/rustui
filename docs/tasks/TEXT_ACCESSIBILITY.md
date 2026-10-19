# Text accessibility

RustUI `Text` and `LocalizedText` expose a GPUI `Label` accessibility role and
use the resolved visible string as the accessible name. A literal supplies its
value directly; `LocalizedText` resolves its message in the active localization
context before both rendering and naming. Font weight, size, and caller-supplied
fallback policy continue through the existing text lowering path.

## Identity

Each label receives an element ID from its owning component and its position
within that render. Environment scopes carry the position count forward, so
two equal strings in the same component remain separate nodes. This is
positional identity for unkeyed text; conditional insertion or reordering can
change a later label's ID. There is no public explicit stable `Text` ID API.

## Bounded native observation

The M48 review used
`target/headstart/verification/M46 M48 Review.app/Contents/MacOS/rustui-playground`
(SHA-256 `2ef11c0a14ae8d8f2b7319850ae6eb85e51d7d3c46943d8461c9ae19d6de6dcf`).
The root agent observed literal `Text` labels in the macOS accessibility tree.
Exactly two distinct nodes, 65 and 66, had the name `Repeated text label`; one
was inside an Environment scope. Localized node 47 changed from `Close` to
`Fermer` to `إغلاق`, and node 32 changed from `Locales` to `Langues` to
`اللغات` as the locale moved through English, French, and Arabic. Both
repeated labels remained present through those changes. The bounded record is
`target/headstart/milestone48-text-accessibility-gui.json`.

The recorded workspace suite at
`target/headstart/milestone46-48-tests-label-fixed.log` reports 132 passed,
0 failed across 12 result groups. Native screen reader announcements, identity
under arbitrary conditional reordering, and full keyboard accessibility have
not been accepted.
