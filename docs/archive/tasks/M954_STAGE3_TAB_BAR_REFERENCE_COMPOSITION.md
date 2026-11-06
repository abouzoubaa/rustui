# M954 — Stage 3 / compose the Tab Bar reference with public TabView

## Gap

The `tab_bar` catalog key already routes to the generic Tabs framework story, but
`catalog_demo("tab_bar")` returned no composition. RustUI exposes `TabView` and
`TabItem`, and the compiled preview already has a TabView specimen, so this
source reference can be represented through the public API without adding a
renderer or changing upstream dependencies.

## Change

- Added a Playground Tab Bar composition with Overview, Activity, and Settings
  tabs, a stable view identity, an explicit accessibility group label, and
  owner-bound selection.
- Routed the `tab_bar` catalog key to the composition.
- Added a focused test for catalog reachability, initial selection, accepted
  selection updates, and rejection of an unknown selection value.

## Validation

`./scripts/cargo-headstart test -p rustui-playground --lib tab_bar_catalog`
passed both matching tests. The build reports the existing unused
`Coverage::Missing` variant warning. Native visual, keyboard, and accessibility
acceptance remains pending.
