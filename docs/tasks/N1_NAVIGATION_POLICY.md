# N1: Navigation controls and unavailable destinations

## Edits

- `crates/rustui/src/components/navigation/navigation_stack.rs` renders NavigationLink and Back through the existing RustUI `Button` using `AppAction::new` and the owner-current `binding_updater`. The link keeps its explicit ID and disabled setting; Back keeps its derived ID. The matching RustUI button styles retain their former secondary and ghost appearances while using RustUI theme metrics and accessibility defaults.
- `locales/{en,fr,ar}/rustui.ftl` adds simple messages for Back and unavailable destination status. The route is rendered as separate text because the catalog parser accepts simple messages only.

## Behavior and API

An empty path shows the root. A known last route shows its destination. An unknown last route shows the localized unavailable status followed by the route string. The bound path is retained, and Back removes one route using the current owner value. Rendering never changes the path. Public constructors, IDs, and disabled behavior are unchanged. This is a visible fallback for missing destinations, not a new route resolver or navigation engine.

Navigation destination IDs and link routes must be nonempty and non-whitespace. Constructors reject blank values before they can enter the bound path or derive a blank link identity.

## Dependencies and limits

Navigation views → RustUI `Button` → `AppAction` → `RenderContext::binding_updater` → GPUI component owner. Locale messages come from `RenderContext::localized_message`, which returns the key if no active localization exists. Translation wording needs native-speaker review. GUI checks remain pending for button appearance, focus and accessibility announcements, missing destination text in all three locales, right-to-left layout, and Back behavior after rapid path changes.

## Acceptance status

M380 route-ID validation is covered by focused locked/offline tests (6/6). Native navigation and accessibility acceptance remain open.


## M99 — native keyboard acceptance (2026-10-19)

The canonical Playground check covered Up/Down, Home/End, disabled Archive, and Return. Enabled destinations changed immediately with navigation keys. Keyboard navigation and a click left the disabled Archive unselected. Return preserved the current selected ID and detail, matching `docs/PUBLIC_API.md`'s contract that Enter has no separate activation callback. Evidence: `target/headstart/evidence/navigation-split-keyboard-acceptance-20261019.md`.


## M511 — validate NavigationLink retained identity

`NavigationLinkView::id(...)` rejects empty and whitespace-only retained
identities before passing them to the underlying Button, and preserves
nonblank caller values verbatim. The focused `navigation_stack` suite passed
9/9. Native link focus, VoiceOver, route-transition behavior, and
cross-platform acceptance remain open. Evidence:
`target/headstart/evidence/m511-navigation-link-identity-tests-20261021.log` and `target/headstart/evidence/rustui-lib-regression-after-m511-20261021.log`.


## M513 — disambiguate repeated NavigationLink routes

Links without explicit IDs now use the render owner's implicit sibling slot in
their fallback identity. Repeated links to the same route therefore no longer
share a retained Button ID; explicit caller IDs are preserved. The
`navigation_stack` suite passed 9/9, the sibling-slot identity regression
passed 1/1, and the full RustUI library suite passed 583/583. No native
duplicate-link focus or activation probe was run. Evidence:
`target/headstart/evidence/m513-navigation-link-sibling-identity-tests-20261021.log`,
`target/headstart/evidence/m513-implicit-sibling-identity-tests-20261021.log`,
and `target/headstart/evidence/rustui-lib-regression-after-m513-20261021.log`.

## M622 — Duplicate destination route IDs

`NavigationStackView::new` requires unique route IDs for all registered destinations. A focused regression now verifies duplicate IDs panic with the constructor's established message. The targeted test passed 1/1; this does not close broader navigation acceptance.
