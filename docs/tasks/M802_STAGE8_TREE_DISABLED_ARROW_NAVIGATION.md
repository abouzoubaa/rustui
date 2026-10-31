# M802 — Stage 8 Tree disabled-row arrow navigation

Tree Up/Down navigation moves among enabled visible entries and wraps at the
ends. When the current entry is disabled, its actual visible position anchors
the next move. Directional navigation keeps the existing scroll-to-entry
behavior; when every entry is disabled, the key event is not consumed.

The focused regression `tree_arrow_navigation_skips_disabled_entries_and_wraps`
passed (1 passed, 761 filtered out). This is source-level keyboard evidence;
native focus and accessibility acceptance remain open.
