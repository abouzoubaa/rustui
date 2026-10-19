# C3: Icon and Label

## Public API

- `components::basic::icon::{Icon, IconName, IconView}` creates a named icon
  from GPUI Components' pinned `IconName` catalog. `.size(px)` validates and
  sets an explicit logical-pixel size; `.accessibility_label(text)` names a
  standalone meaningful icon.
- `components::basic::label::{Label, LabelView}` creates a GPUI Components
  semantic text label. `.secondary(text)` uses its muted secondary treatment;
  `.icon(name)` places a decorative icon before the visible label with a 6px
  gap. The visible text remains the accessible name.

`IconName` is re-exported from GPUI Components 0.7.3 and uses that component
enum directly; it is the pinned component catalog, not the broader
`gpui_kit_assets::IconName` catalog. No string-to-icon guessing, image
placeholder, custom asset source, or fallback glyph is added. The pinned Icon
inherits current text color, and Label uses GPUI Components' current semantic
text styling.

## Verification boundary

Focused unit tests cover icon size validation and label option construction.
Workspace compilation, visual rendering, and accessibility tree inspection
remain with the root task.
