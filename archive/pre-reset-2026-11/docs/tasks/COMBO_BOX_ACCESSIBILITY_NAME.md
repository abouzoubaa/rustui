# ComboBox accessible name

`ComboBox` uses its constructor label as the visible placeholder and, by
default, as the accessible name forwarded to the pinned GPUI Components Select.
The constructor rejects labels containing only whitespace and preserves
accepted labels verbatim. `.localized_accessibility_label(message_id)` resolves
the accessible name through the active `RenderContext` catalog, validates that
the message ID is nonblank, and falls back to the constructor label when the
catalog result is blank or unresolved. Focused source tests cover these
behaviors in `crates/rustui/src/components/input/combo_box.rs`.

This is source-level validation only. Existing bounded ComboBox interaction
evidence is recorded in `docs/tasks/C10_COMBO_BOX_SELECT.md`; native accessible
name announcement, full keyboard and focus behavior, VoiceOver, RTL,
cross-platform, and visual acceptance remain open.
