# Image accessible-name validation

Content images can be named with `Image::accessibility_label`. The setter
rejects empty or whitespace-only names. `.localized_accessibility_label`
resolves a message ID through the active RustUI catalog. Because images have
no visible text, callers provide the explicit accessible name first; it remains
the fallback when the catalog returns a missing, blank, or ID-echo value. Omit
the setter for decorative images; they remain unannounced.

Focused unit coverage verifies explicit-name retention, blank-name and
blank-message-ID rejection, catalog resolution, and fallback behavior. Native
accessibility-tree and screen-reader acceptance remain open.
