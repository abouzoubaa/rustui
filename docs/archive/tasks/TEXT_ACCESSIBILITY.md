# Text accessibility

RustUI `Text` and `LocalizedText` expose a GPUI `Label` accessibility role and
use the resolved visible string as the accessible name. A literal supplies its
value directly; `LocalizedText` resolves its message in the active localization
context before both rendering and naming. Font weight, size, and caller-supplied
fallback policy continue through the existing text lowering path.

## Identity

Each label receives an element ID from its owning component and its position
within that render unless the caller supplies `Text(...).id(...)`. Environment
scopes carry the position count forward, so two equal strings in the same
component remain separate nodes. This is positional identity for unkeyed text;
conditional insertion or reordering can change a later label's ID. The explicit
ID is stable within the render owner, geometry reader, or grid cell contract.

`LocalizedText(...).with_arguments(...)` resolves named arguments in the active
locale at render time. A resolved blank string remains renderable, but RustUI
omits its explicit GPUI accessibility label so it does not create an empty
accessible name.

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

## Input clear action

Cleanable text inputs expose a clear icon only when the field is editable and
has a value. RustUI supplies the button's accessible name as `Clear {placeholder}`
and clears the owner binding on activation. This replaces GPUI Components'
icon-only clear button, which has neither visible button text nor an explicit
accessible label. SearchField enables this action by default; callers can use
`cleanable(false)` to omit it.
