# Icon accessible-name validation

`Icon::accessibility_label` rejects empty and whitespace-only names while
retaining nonblank text exactly as supplied. `.localized_accessibility_label`
resolves its message ID through the active RustUI catalog. Since icons have no
visible text, callers set an explicit accessible name first; that name remains
the fallback for missing, blank, or ID-echo catalog results. Decorative icons
can omit the override; labeled icons expose the resolved name through their
image-role wrapper.

Focused source tests cover blank literal and message-ID rejection, catalog
resolution, and fallback behavior. They do not establish native
accessibility-tree or screen-reader acceptance.
