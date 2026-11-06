# TextField accessible-name validation

`TextField` permits an empty placeholder. An explicit `accessibility_label(...)`
override must contain at least one non-whitespace character and is retained
exactly as supplied. `localized_accessibility_label(...)` accepts a nonblank
message ID and resolves it through the active `RenderContext` catalog. The
literal override becomes the localized label's fallback; otherwise the field
uses its placeholder, then the generic `Text field` name when the placeholder
is blank. An unresolved or blank catalog result uses that same fallback.

Focused Rust unit tests cover blank literal and message-ID rejection, literal
preservation, localized resolution, and fallback behavior. These checks do not
establish native accessibility-tree, screen-reader, or cross-platform
acceptance.
