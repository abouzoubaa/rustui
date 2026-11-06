# C7: Badge and Avatar adapters

## Public API

- `Badge(content)` decorates content with a GPUI Components badge. `.count(n)`
  displays a number (zero hides it), `.dot()` displays a dot, `.max(n)` uses
  the native `n+` cap, and `.size(BadgeSize)` selects Small, Medium, or Large.
- A visible badge gets a GPUI Status role and a default accessible label such
  as “7 notifications” or “Notification available”. `.accessibility_label(text)`
  overrides it; `.id(identity)` supplies a stable ID. The default ID derives
  from badge kind/count, so identical sibling badges need distinct explicit
  IDs. The status wrapper contains the native badge and content; it does not
  replace the content with an image role.
- `Avatar(name)` uses GPUI Components' initials fallback. Whitespace is
  trimmed and collapsed before the native component derives initials; a blank
  name uses the native anonymous placeholder. `.source(ImageSource)` supplies
  an embedded asset or filesystem image, and `.size(px)` sets a custom diameter.
- Avatars receive the GPUI Image role and an accessible name, defaulting to the
  normalized name or “User avatar”. `.accessibility_label(text)` overrides the
  name; `.id(identity)` sets the stable element ID. The default ID derives from
  the normalized name; identical siblings need distinct explicit IDs.
- Avatar loading, initials rendering, fallback icon, coloring, and clipping all
  remain native GPUI Components behavior.

Custom avatar diameters must be finite and positive. Badge sizing uses the
component's named size enum, so no unchecked numeric dimension is accepted.
Image sources use the RustUI `ImageSource` boundary and GPUI's existing image
loader/cache.

## Limits and verification boundary

These adapters do not add custom badge colors/icons, avatar groups/status,
remote URL interpretation, or image fallback drawing. Focused tests cover
avatar name normalization and size validation, plus badge label and ID
resolution.

## Native acceptance (M19)

The native Playground review showed the Ada Lovelace initials “AL” and its image
name in the accessibility tree. The count badge showed “7”; activating its child
button changed the action count from 0 to 1. Evidence:
`target/headstart/milestone19-links-gui.json`.

These observations cover the exercised initials and count-badge cases. They do not
accept other avatar image/loading paths, badge variants, visual states, or screen-reader
announcements. Compilation and broader acceptance are not claimed by this task record.
