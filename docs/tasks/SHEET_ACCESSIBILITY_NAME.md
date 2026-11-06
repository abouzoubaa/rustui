# Sheet accessible name

An explicit literal `.accessibility_label(...)` must contain non-whitespace
text; accepted text is preserved verbatim. When no explicit name is supplied,
Sheet continues to use its visible title, then trigger label. The
`.localized_accessibility_label(...)` rejects whitespace-only message keys.

Focused source tests for blank rejection, literal preservation, and the existing
fallback order live in `crates/rustui/src/components/overlay/sheet.rs`.

This is source-level validation only. Native Sheet host and close-button
semantics, screen-reader behavior, theme/locale, RTL, cross-platform, and visual
acceptance remain open.

## Localized-trigger fallback regression

When no explicit accessible name or title is supplied, `.localized_trigger(key)` resolves the active catalog value as the accessible-name fallback. The focused Sheet regression checks that this candidate remains a message key in the test model, matching the render-time resolver. Workspace library tests passed 630/630; native locale-switching and screen-reader acceptance remain open. Evidence: `target/headstart/evidence/sheet-localized-fallback-contract-20261021.log`.

## Localized accessible-name key validation (2026-10-21)

`SheetView::localized_accessibility_label` now rejects whitespace-only message
keys, matching the explicit literal accessible-name and localized-trigger
builders. This prevents an invalid explicit name from being silently skipped by
the resolved-name fallback. The focused Sheet suite passed 15/15; the locked,
offline RustUI all-targets check passed with existing example warnings. Evidence:
`target/headstart/evidence/sheet-localized-name-key-contract-20261021.log` and
`target/headstart/evidence/overlay-localized-name-key-rustui-check-20261021.log`.
Native host, screen-reader, and focus acceptance remain open.
