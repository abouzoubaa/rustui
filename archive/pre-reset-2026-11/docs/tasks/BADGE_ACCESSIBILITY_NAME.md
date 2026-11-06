# Badge accessible-name validation

`Badge::accessibility_label` rejects empty and whitespace-only overrides so a
blank override cannot hide the generated count or dot announcement. Nonblank
overrides are retained exactly as supplied. `localized_accessibility_label`
accepts a nonblank catalog message ID and falls back to the explicit label or
the visible badge announcement when the message is missing, blank, or unresolved.
A hidden zero-count badge still exposes no status.

Focused source tests cover blank literal labels and message IDs, literal
preservation, localized-ID/fallback storage, generated announcements, and the
hidden zero-count state. They do not establish native accessibility-tree or
screen-reader acceptance.
