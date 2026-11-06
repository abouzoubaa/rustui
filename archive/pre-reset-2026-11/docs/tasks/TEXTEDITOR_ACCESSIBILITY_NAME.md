# TextEditor accessible-name contract

`TextEditor` supports literal and localized accessible names. Literal overrides
must contain at least one non-whitespace character and remain exact, including
surrounding spaces. `localized_accessibility_label` validates its message ID and
resolves it through the active `RenderContext` catalog. A missing or blank
translation falls back to the explicit literal name, then the visible
placeholder, then `Text editor` when the placeholder is blank. Existing views
without an explicit or localized override use the visible placeholder, or the
`Text editor` default when the placeholder is blank.

Focused source tests cover literal preservation, catalog resolution, missing
and blank translation fallback, explicit-name precedence, the default fallback,
and blank message-ID rejection. These checks do not establish native
accessibility-tree, screen-reader, or cross-platform acceptance.

# M1193 — default TextEditor accessible name

The render path now always applies the resolved accessible name to the
underlying Textarea. This supplies `Text editor` when no override is configured
and the placeholder is blank, where the upstream placeholder-derived fallback
would otherwise leave the control unnamed. The focused default-name regression
passed 1/1. Native accessibility-tree, screen-reader, and cross-platform
acceptance remain open.
# M405 — refresh retained TextEditor binding
TextEditor now routes retained change events through the latest owner `Binding` after a same-identity rerender. The focused module suite passed 3/3, including a latest-binding regression. Native multiline interaction, focus, IME, and accessibility acceptance remain open. Evidence: `target/headstart/evidence/m405-text-editor-focused-final.log`.
