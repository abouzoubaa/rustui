# M1039 — Localize the semantics gallery Region name

## Problem

The Playground semantics example exposed a Region named with the localized
Close action label. That name did not describe the Region's content.

## Change

Added a dedicated `rustui-semantics-example-region` label to the shipped
English, French, and Arabic catalogs and used it for the example Region.

## Focused evidence

`./scripts/cargo-headstart test --locked --offline -p rustui-i18n --test catalog_contract semantics_gallery_region_accessibility_name_is_localized`

Result: 1 passed, 0 failed. This verifies the three catalog values, not the
rendered Playground accessibility tree or screen-reader speech.
