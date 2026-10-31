# M1188 — Stage 3 utility-window title fidelity

## Scope

Correct the Playground `utility_window` preview header to match the catalog's
“Utility Panel” description. No component API, runtime behavior, or native
window integration changes.

## Change

`apps/rustui-playground/src/stories/utility_window_gallery.rs` now renders
“Utility Panel” instead of the placeholder “Title.”

## Verification

Scoped `git diff --check` passed. This copy-only correction does not establish
native visual, keyboard, accessibility, localization, or platform acceptance.
