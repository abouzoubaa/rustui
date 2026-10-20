# Semantics groups (M51)

`Semantics(id, child)` gives a RustUI subtree a stable structural identity. Set `.role(SemanticsRole::Group | Region | Section | Navigation)`, `.label(...)`, `.localized_label(...)`, `.description(...)`, or `.localized_description(...)` to expose a native accessibility node. A label or description without an explicit role uses `Group`. With no metadata, the wrapper creates no accessibility node. Localization resolves against the active catalog each render.

The wrapper delegates the child's layout, prepaint, and paint to GPUI, preserving the child's own accessibility nodes, roles, and names. The group label names the group, not a descendant control. The ID must be nonempty and unique among siblings in the same logical scope; repeated sibling IDs panic during rendering. Its native element ID uses a dedicated `semantics` namespace.

This milestone supports structural roles and AccessKit live status values. `live_region_value` and `localized_live_region_value` expose the changing value and announcement politeness, and assign the Status role unless a later `.role(...)` overrides it; the Playground counter is the public-API example. Prefer polite announcements for routine status changes. It does not provide interactive roles, actions, focus callbacks, or accessibility-only hiding. Hiding a subtree from accessibility while leaving focusable descendants in keyboard navigation needs a coordinated focus policy and is deferred.

## Checks

- Unit checks cover role mapping, live value/politeness writing, blank IDs, duplicate sibling scope IDs, and native namespace separation.
- Cargo and native GUI validation are owned by the root task and are not recorded here.

## Executed verification

The final workspace suite passed 143 tests across 12 binaries, zero failures
(`target/headstart/milestone51-tests-fixed.log`). The initial integration failure
in `milestone51-tests.log` is retained; its redundant demo was removed.
Final Playground build and pinned dependency artifact reuse passed.
Native review exposed the named group and description, preserved its child Text
and Button, observed Increment 0→1, and changed the region name Close→Fermer→إغلاق
without losing the counter. The no-metadata wrapper exposed its child without an
extra accessibility container. Evidence and binary hash:
`target/headstart/milestone51-semantics-gui.json`.

This is bounded accessibility-tree and pointer acceptance. The new live-region
source contract is covered by unit checks; screenreader announcements, keyboard
traversal, description localization, reordered native node identity and wider
platform support remain unverified.
