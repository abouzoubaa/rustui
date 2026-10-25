# Semantics groups (M51)

`Semantics(id, child)` gives a RustUI subtree a stable structural identity. Set `.role(SemanticsRole::Group | Region | Section | Navigation | Status | Alert)`, `.label(...)`, `.localized_label(...)`, `.description(...)`, or `.localized_description(...)` to expose a native accessibility node. A label or description without an explicit role uses `Group`. With no metadata, the wrapper creates no accessibility node. Localization resolves against the active catalog each render.

The wrapper delegates the child's layout, prepaint, and paint to GPUI, preserving the child's own accessibility nodes, roles, and names. The group label names the group, not a descendant control. The ID must be nonempty and unique among siblings in the same logical scope; repeated sibling IDs panic during rendering. Its native element ID uses a dedicated `semantics` namespace.

This milestone supports structural roles, AccessKit live status values, and the Alert role for important dynamic errors. `live_region_value` and `localized_live_region_value` expose the changing value and announcement politeness, and assign the Status role unless a later `.role(...)` overrides it; the Playground counter is the public-API example. `.role(SemanticsRole::Alert)` exposes Alert semantics; the Playground includes a save-error example. Prefer polite announcements for routine status changes. It does not provide interactive roles, actions, focus callbacks, or accessibility-only hiding. Hiding a subtree from accessibility while leaving focusable descendants in keyboard navigation needs a coordinated focus policy and is deferred.

## Checks

- Unit checks cover role mapping, live value/politeness writing, blank IDs, duplicate sibling scope IDs, and native namespace separation.
- Localized label, description, and live-region builders reject whitespace-only message keys so they cannot resolve to empty accessibility text. The focused semantics suite passed 8/8 on 2026-10-21 (`target/headstart/evidence/semantics-localized-message-key-test-20261021.log`).
- M482 adds a named `SemanticsRole::Navigation` landmark to the existing
  Playground semantics gallery. The Navigation builder regression passed as
  part of the focused semantics suite (9/9), and the locked/offline Playground
  all-targets check passed. The first native probe searched the catalog and did
  not select the existing Text framework story that mounts this gallery; the
  follow-up selected Text and captured the gallery in the native AX tree. The
  tree exposed a `navigation` node, confirming the role, but emitted no node
  name; its child text was `Navigation landmark example`. The accessible name
  `Primary navigation` is therefore not confirmed by this snapshot. VoiceOver
  announcements were not tested. Evidence:
  `target/headstart/evidence/navigation-semantics-builder-m482-test-20261021.log`,
  `target/headstart/evidence/navigation-semantics-playground-m482-check-20261021.log`,
  `target/headstart/evidence/m482-navigation-landmark-native-probe-20261021.json`,
  and `target/headstart/evidence/m482-navigation-landmark-native-followup-20261021.json`.
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

## M529 literal label validation

`SemanticsView::label(...)` now rejects empty and whitespace-only literal labels, matching the localized-label key validation. Nonblank labels retain their supplied text. The focused Semantics suite passed 11/11 in `target/headstart/evidence/m529-semantics-literal-label-validation-tests-20261021.log`. Native accessibility-tree and VoiceOver acceptance remain unverified.

## M552 literal description validation

Literal empty and whitespace-only descriptions are rejected before AccessKit serialization. Nonblank text is preserved. Semantics tests passed 13/13 in `target/headstart/evidence/m552-semantics-literal-description-validation-20261021.log`. Native accessibility-tree and VoiceOver acceptance remain unverified.


## M559 literal live-region value validation
`SemanticsView::live_region_value` preserves nonblank text verbatim, allows the empty string to clear an existing announcement, and rejects whitespace-only values. Focused Semantics tests passed 14/14 in `target/headstart/evidence/m559-semantics-live-region-value-validation-20261021.log`. AccessKit and native VoiceOver acceptance remain unverified.
