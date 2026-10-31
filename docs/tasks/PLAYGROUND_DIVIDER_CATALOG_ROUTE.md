# M829 — Playground Divider catalog route

Stage 9: route the catalog's implemented Divider specimen to its existing demo.

The full display catalog has 46 entries. Its coverage enum contains only
`Partial` and `Missing`; these values do not promise a dedicated catalog route.
Before this change, `catalog_demo` handled 19 keys. The Divider note explicitly
states that horizontal and vertical Divider are implemented and visually
checked, but Divider had neither a catalog route nor a `story_for` fallback,
so selecting it displayed the pending-demo message.

`catalog_demo("divider")` now reuses `foundation_additions::foundation_additions`,
which already composes horizontal and vertical Divider examples. No specimen,
adapter, dependency, or catalog status was added.

Validation: `./scripts/cargo-headstart test -p rustui-playground --lib
divider_catalog_reaches_horizontal_and_vertical_demo` passed: 1 passed, 0 failed,
123 filtered out. This verifies route availability; no GUI check ran.
