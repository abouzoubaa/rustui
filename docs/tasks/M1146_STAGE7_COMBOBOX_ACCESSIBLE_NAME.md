# M1146 — Stage 7: support explicit accessible names on ComboBox

ComboBox permits a blank visible label when the caller supplies a non-empty
explicit accessible name. The localized-name fallback preserves that explicit
name instead of replacing it with a placeholder.

Focused ComboBox tests passed 9/9. Native screen-reader and locale-switching
acceptance remain open.
