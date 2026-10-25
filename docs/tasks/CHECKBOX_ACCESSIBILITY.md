# Checkbox accessibility probe

The canonical Playground **Text inputs** story keeps an enabled `Accepted`
checkbox and a disabled `Accepted (disabled)` probe side by side. The disabled
probe has the explicit accessible name `Accepted option (disabled)` and shares
the enabled checkbox's owner binding, so any accidental activation is visible
on both controls.

Native acceptance remains pending. On a fresh canonical Playground build,
record each control's native checkbox role, accessible name, and checked value;
verify visible focus and Space/Return activation on the enabled checkbox; then
verify the disabled probe is skipped by keyboard focus and does not change the
shared value through pointer or keyboard input. Do not treat source forwarding
or the pinned GPUI Base unit test as native acceptance.

## Localized accessible name

`CheckboxView::localized_accessibility_label` resolves its message ID through
the active RustUI catalog at render time. The visible checkbox label is the
fallback unless an explicit `.accessibility_label(...)` override was supplied;
that fallback survives later localized-label changes. A missing, blank, or
unresolved catalog result uses the fallback. Blank message IDs are rejected.
Focused source tests cover fallback preservation, catalog value selection, and
blank-ID rejection. Native screen-reader verification remains pending.

## M368 — Disabled accessibility state

The RustUI Checkbox adapter now also sets the AccessKit disabled state when
`.disabled(true)` is supplied. The pinned GPUI Base checkbox suppresses focus
and activation for disabled controls but does not set this state itself. A
focused source regression verifies the AccessKit disabled flag; native
accessibility-tree and screen-reader acceptance remain pending.
