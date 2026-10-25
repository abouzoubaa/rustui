# Stage 9 / D1c ScrollView preview specimen

## Scope

The compiled preview gallery registers an opt-in `scroll-view-specimen` using
the public RustUI `ScrollView` API. Its descriptor exposes a bounded label,
the `vertical` / `horizontal` / `both` axis choice, and scrollbar visibility.
The compiled-local factory provides fixed text rows for each axis mode and
owns the stable viewport ID and dimensions. Preview data does not serialize
views, callbacks, closures, or scroll state.

## Validation

- Focused gallery suite passed 56/56 tests, including descriptor round-trip
  coverage for the label, both-axis selection, and scrollbar setting.
- Locked/offline Playground all-targets check passed. The existing
  `mount_property_inspector_specimen` dead-code warning and upstream `block`
  future-incompatibility warning remain.
- `rustfmt --edition 2024 --check` and scoped `git diff --check` passed.
- Evidence: `target/headstart/evidence/d1c-scroll-view-specimen-tests-20261021.log`
  and `target/headstart/evidence/d1c-scroll-view-specimen-check-20261021.log`.

No GUI acceptance was run. Native scroll input, appearance, accessibility,
RTL behavior, and cross-platform acceptance remain open.
