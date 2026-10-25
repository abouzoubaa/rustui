# Avatar accessible-name validation

The explicit `AvatarView::accessibility_label` override rejects whitespace-only
text and preserves nonblank input verbatim. Omitting the override uses the
normalized avatar name, or `User avatar` when the constructor name is blank.
The explicit `.id(...)` override rejects whitespace-only identity values so
invalid IDs do not reach GPUI; nonblank identities are preserved.
`localized_accessibility_label` accepts a nonblank catalog message ID and uses
the explicit label or normalized avatar name as its fallback when the message
is missing, blank, or unresolved.

Focused tests cover blank literal labels and message IDs, literal preservation,
localized-ID/fallback storage, name normalization, and the anonymous fallback.
These source tests do not establish native accessibility-tree or screen-reader
acceptance.
