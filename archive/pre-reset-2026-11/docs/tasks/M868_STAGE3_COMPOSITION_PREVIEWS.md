# M868 — Stage 3 primitive-composition catalog previews

## Change

The Playground catalog routes `utility_window`, `form`, `menu_bar`, and
`window_controls` to visible compositions built from existing RustUI
primitives. These entries no longer render the generic pending placeholder.
The window-controls composition is labeled as illustrative and does not claim
to provide native operating-system window chrome.

## Focused evidence

- `primitive_composition_catalog_entries_reach_previews` passed: 1 passed,
  126 filtered.
- `git diff --check` passed.

## Acceptance boundary

This milestone supplies bounded catalog compositions. It does not establish
native window-chrome support, broad visual acceptance, or accessibility
acceptance for these previews.
