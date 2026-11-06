# Stage 3 — Segmented Control reference composition

## Contract

The catalog's `segmented_control` entry references the List and Unified Compact
Toolbar + Title artwork and routes to the Tabs story. RustUI has no dedicated
Segmented Control renderer, but its public `Button` supports persistent toggled
state and public styles, so the Playground can demonstrate the Finder view
modes without adding a framework API.

## Change

The catalog entry now routes to four mutually exclusive public Buttons for
Icons, List, Columns, and Gallery. The selected mode is shown below the row,
and the catalog note describes this as a Button composition rather than a
dedicated control implementation.

## Validation and limits

The focused regression checks that each of the four actions selects exactly
one mode and updates the displayed mode label. The check is
`./scripts/cargo-headstart test -p rustui-playground --lib segmented_control`;
it passed with 3 tests, 0 failures, and 180 filtered out. This is source-level
composition evidence. Native visual, keyboard, and accessibility acceptance
remains pending.
