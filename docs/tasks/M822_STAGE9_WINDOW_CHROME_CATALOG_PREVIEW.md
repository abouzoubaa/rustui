# M822 — Stage 9 WindowChrome catalog preview

## Finding

The Playground catalog marks WindowChrome as a partial composed preview, but selecting its entry fell through to the generic pending placeholder.

## Change

`catalog_demo("window_chrome")` now returns a composition using the existing Toolbar gallery between a title and content label. This reuses the existing Playground story and adds no new framework component or dependency.

## Evidence

- Regression: `apps/rustui-playground/src/stories/mod.rs::window_chrome_catalog_routes_to_composed_preview`
- Command: `./scripts/cargo-headstart test -p rustui-playground --lib window_chrome_catalog_routes_to_composed_preview --locked --offline`
- Result: 1 test passed.
- Scoped `git diff --check` passed.

## Remaining acceptance

This verifies the catalog route only. Native visual and accessibility acceptance remain open.
