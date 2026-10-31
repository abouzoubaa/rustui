# D1c TabView keyboard and accessibility observation

## M609 — Localized per-tab accessible names

`TabItem::localized_accessibility_label` resolves a message ID through the active RustUI locale catalog. An unresolved, blank, or key-echo result falls back to the explicit literal name when present, otherwise to the visible tab label. Calling `.accessibility_label(...)` afterward replaces the localized name; calling localization after a literal keeps that literal as fallback. The tab and its selected `TabPanel` use the same resolved accessible name. Focused TabView tests passed 21/21. The canonical Tabs story demonstrates English, French, and Arabic names; catalog parity passed 1/1, locale validation passed, and the locked/offline Playground all-targets check passed. Native AX-tree exposure, VoiceOver, live locale switching, RTL, and cross-platform acceptance remain open. Evidence: `target/headstart/evidence/m609-tabview-localized-label-test.log`, `target/headstart/evidence/m609-playground-localized-tab-catalog-test.log`, `target/headstart/evidence/m609-locale-validation.log`, and `target/headstart/evidence/m609-playground-all-targets-check.log`.

## M282 — Stage 9 / D1c

The rebuilt disposable Playground exposed a native tab group with the names
Overview, Activity, and Settings and a selected state (`Overview=on`). The
window-level focus report remained at the Playground root. Sending Tab and
Right left the owner selection at `overview` and the visible content unchanged.
The native AX tree therefore exposes tab naming and selection state, but this
run did not establish keyboard focus or activation for the tabs.

This is negative acceptance evidence, not a pass. Pinned GPUI Base documents
that tabs do not participate in keyboard focus and leaves roving focus, arrow
keys, Home/End, and Enter/Space as TODOs in
`vendor/gpui-base-0.7.3/src/tabs.rs`. RustUI must add and validate its own
adapter before this gate can pass; changing pinned upstream sources or
revisions is outside this task.

Keyboard, screen-reader speech, locale, theme, RTL, and cross-platform
acceptance remain open. Native observation record:
`target/headstart/evidence/m282-tabview-keyboard-ax-20261020.json`.

## M284 — RustUI automatic-activation keyboard adapter

RustUI now places a retained, named focus target around the TabView tab list.
When that target itself has focus, unmodified Left/Right select the previous or
next enabled tab, Home/End select the first or last enabled tab, and horizontal
direction follows the active locale. Arrow selection activates immediately.
Tab/Shift-Tab remain available to leave the group; keys are ignored while IME
or character input is preferred, and selection keys in focused tab content are
not intercepted. Pointer activation moves focus to the group.

This is a RustUI-owned keyboard first slice, not full tab primitive completion.
The pinned TabBar does not expose per-tab focus handles, so this adapter cannot
provide per-tab roving focus or manual Enter/Space activation. Native focus,
selected-state announcements, tab-to-panel traversal, disabled-tab behavior,
RTL behavior, VoiceOver, and cross-platform checks remain open. Focused
helper/build evidence is `target/headstart/evidence/m284-tabview-keyboard-test-20261020.log`
and `target/headstart/evidence/m284-tabview-keyboard-check-20261020.log`.


Native M284 interaction record: `target/headstart/evidence/m284-tabview-keyboard-native-20261020.json`.
It observed keyboard selection and disabled-tab skipping in the Playground. The
native tree still reported focus at the window root, so native focus and spoken
announcement acceptance remain open.

## M285 — Stable TabView group name

The TabView focus group now uses the stable fallback label `Tabs` instead of
reusing the selected tab's label. Callers can still provide a descriptive group
name with `.accessibility_label(...)`; the selected tab retains its own name.
Focused tests passed 8/8 and the locked/offline Playground all-targets check
passed. This corrects group naming at source level; the native tree still
reported window-root focus in M284, so focus and screen-reader acceptance remain
open. Evidence: `target/headstart/evidence/m285-tabview-stable-group-name-test-20261020.log`
and `target/headstart/evidence/m285-tabview-stable-group-name-check-20261020.log`.

## M322 — Per-tab accessible-name override

`TabItem::accessibility_label` now allows callers to keep a concise visible tab
label while supplying a more descriptive accessible name. Tabs without an
override continue to use their visible label, and blank overrides are rejected.
Focused TabView tests passed 10/10 and the locked/offline Playground all-targets
check passed. Native accessibility-tree, screen-reader, keyboard focus,
localization, RTL, and cross-platform acceptance remain open. Evidence:
`target/headstart/evidence/m322-tabview-accessible-name-test-20261020.log` and
`target/headstart/evidence/m322-tabview-accessible-name-check-20261020.log`.

## Current source follow-up — roving focus and manual activation (2026-10-21)

RustUI now attaches retained focus handles to individual tabs through the
pinned `gpui_component::tab::Tab` public interactive-element API. Exactly one
enabled tab is in the Tab order; Left/Right (reversed in RTL) and Home/End move
focus without changing selection, and Enter/Space activate the focused tab.
Pointer activation focuses and selects the clicked tab. Disabled tabs are
skipped, Tab traversal is left alone, and keys from focused tab content are not
handled by the tab-list adapter.

The pinned `TabBar` wrapper exposes tab roles, selected/disabled state, and
accessible labels, but it does not expose the underlying tab's position-in-set
setter or a tab-to-panel ID relationship. Native focus, spoken announcements,
tab-to-panel association, localization, theme, RTL, and cross-platform
acceptance remain open. Focused source tests passed 12/12 and the locked/offline
Playground all-targets check passed. The check reports three existing unused
specimen methods in `compiled_preview_gallery.rs`; no GUI run was performed.
Evidence: `target/headstart/evidence/tabview-roving-activation-test-20261021.log`
and `target/headstart/evidence/tabview-roving-activation-check-20261021.log`.

## 2026-10-21 — Stage 8 source contract audit

Reviewed stable tab IDs and container identity, controlled selection and
roving-focus recovery, disabled-tab traversal, and visible/overridden/group
accessible-name validation. No decision-free source defect was found, so no
implementation change was made. Focused source tests passed 12/12 with
`./scripts/cargo-headstart test -p rustui tab_view --lib --locked`. This is
source-level evidence only; native focus, announcements, panel association,
localization, RTL, theme, and cross-platform acceptance remain open.

## M497 — localized tab-group accessible name

The default TabView group name now resolves from active localization using
`rustui-navigation-tabs`; `.accessibility_label(...)` remains the literal
override and `.localized_accessibility_label(message_id)` supports localized
custom names. English, French, and Arabic catalogs have matching entries.
TabView tests passed 12/12, localization catalog tests 39/39, and the
locked/offline Playground all-targets check passed. Native VoiceOver speech and
focus behavior remain acceptance items. Evidence:
`target/headstart/evidence/m497-tab-view-accessibility-tests-20261021.log`,
`target/headstart/evidence/m497-localized-tabs-catalog-tests-20261021.log`, and
`target/headstart/evidence/m497-playground-all-targets-check-20261021.log`.


## M510 — validate TabView container identity

`TabViewControl::id(...)` now rejects empty and whitespace-only identities
before they become retained GPUI state keys, and preserves a valid caller ID
verbatim. The focused TabView suite passed 14/14; the RustUI library suite
passed 579/579. Native focus, VoiceOver, tab-panel association, localization,
RTL, theme, and cross-platform acceptance remain open. Evidence:
`target/headstart/evidence/m510-tabview-identity-tests-20261021.log` and
`target/headstart/evidence/rustui-lib-regression-after-m510-20261021.log`.

## 2026-10-21 — selected panel source semantics

The selected TabView content is now wrapped in a named `Role::TabPanel`.
Its element identity combines the TabView identity and selected stable tab ID,
and its accessible name uses that tab's existing visible-label or explicit
accessible-name resolution. Only the selected content is wrapped/rendered;
inactive tab retention, keyboard selection, and focus behavior are unchanged.
Focused tests cover panel identity stability/scoping and selected-tab name
resolution. This is source-level semantics evidence only: native tab-to-panel
association, screen-reader traversal, and spoken announcements remain
unverified.

## M584 validation

Focused TabView tests passed 17/17; the locked/offline Playground all-targets
check passed. Evidence: `target/headstart/evidence/tabview-panel-semantics-test-20261021.log`
and `target/headstart/evidence/m584-playground-all-targets-check-20261021.log`.
Native tab-to-panel association, focus traversal, VoiceOver, and cross-platform
acceptance remain open.


## M603 — Stage 8 tab position-in-set metadata (2026-10-21)

RustUI now assigns every rendered tab its one-based `position_in_set` and the
total TabView item count as `size_of_set`. Disabled tabs remain included in
these set positions because they are still members of the tab set, while the
existing roving-focus rules continue to skip them. The values are applied at
the public `gpui_component::tab::Tab` construction boundary through pinned
GPUI's `StatefulInteractiveElement` API; no vendor or dependency changes were
needed.

The AccessKit serialization regression verifies a second-of-three tab emits
position 2 and set size 3. The focused test passed 1/1, the full TabView suite
passed 19/19, and the locked/offline RustUI library check passed. This verifies
source-level node metadata only. Native AX exposure, screen-reader
announcements, tab-to-panel association, RTL, and cross-platform acceptance
remain open. Evidence:
`target/headstart/evidence/m603-tabview-tab-set-test-20261021.log`,
`target/headstart/evidence/m603-tabview-suite-20261021.log`, and
`target/headstart/evidence/m603-tabview-tab-set-check-20261021.log`.
