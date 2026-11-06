# M886 — Stage 7 Toggle localized accessible name

## Change

Toggle now accepts a localized accessibility message ID and resolves it
through the active RenderContext, retaining the visible label as fallback
when the translation is missing or blank. The underlying Switch already
exposes checked and disabled state through GPUI Base semantics.

## Focused evidence

- `./scripts/cargo-headstart test -p rustui toggle::accessibility_tests` passed (6 tests).
- Scoped `git diff --check` passed.

## Acceptance boundary

This adds localized accessible-name support and confirms the focused unit
contract. Broader native accessibility, interaction, and cross-platform
acceptance remain open.
