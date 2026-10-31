# NavigationSplitView adaptive collapse

## Implemented source contract

- The container switches to one column below **560 logical pixels**. This
  leaves room above the combined 240-point sidebar preference, 140-point
  sidebar minimum, 100-point detail minimum, and divider while avoiding a
  squeezed two-pane layout on common narrow windows.
- At the first narrow render, a valid selected route opens in the detail view;
  `None` or an unknown ID opens the destination list.
- The detail's **Back** button returns to the destination list and preserves the
  caller-owned selection. A separate **Open selected destination** button
  reopens the selected route from the list, including when the sidebar row is
  already selected.
- Selecting a different valid route while the list is showing opens its detail.
  Clearing selection or selecting an unknown ID shows the list.
- The narrow list/detail mode is retained across width changes. Wide mode always
  uses the existing horizontal `SplitView` and its resizable sidebar. A route
  selection change while wide is reflected when the view next becomes narrow.
- If the selected item is removed while detail mode is active, the narrow view
  returns to the list and leaves the stale ID in the controlled binding. The
  wide view retains its existing empty-detail behavior for an unknown ID.
- The Back button is the first keyboard-focusable control in the detail
  composition. Activating Back restores keyboard focus to the retained Sidebar
  tree container while preserving the caller-owned selected route, so existing
  tree keyboard navigation can continue. Focus is restored to the tree
  container; this does not promise per-row focus because the upstream Sidebar
  API exposes no row focus handle. Retention scopes preserve the Sidebar focus
  handle and detail page state across composition changes.

The implementation uses `GeometryReader` and adds no public API or dependency.
The narrow source-state transitions have focused unit coverage. On 2026-10-21,
the canonical Playground story gained buttons for requested container widths of
480, 560, and 640 logical pixels, so the strict breakpoint can be exercised.
The readout shows the requested width; native acceptance must confirm the actual
rendered bounds. The locked/offline Playground build passed after this change.
Build and formatting record: `target/headstart/evidence/navigation-split-width-probe-build-20261021.json`.

M480's native macOS 27 probe observed the two-pane composition at a requested
640 logical pixels and the one-pane composition at 480. Back/Open preserved the
selected route, and both narrow detail and list modes survived a 480 → 640 → 480
width round trip. At a requested 560 logical pixels the narrow composition
appeared, but the actual rendered width was not measured, so the exact native
threshold remains unverified. Evidence:
`target/headstart/evidence/navigation-split-adaptive-collapse-native-probe-20261021.json`.
Keyboard/focus transfer, screen-reader output, locale, visual fit, and
cross-platform behavior still require acceptance.


## M481 native keyboard and focus probe

On 2026-10-21, the disposable Playground validation app was exercised at requested widths 480 and 640 logical pixels. At 480, Tab showed focus on Back and Shift-Tab moved it to the 640 px width control. Back returned to the destination list with Files selected. Down/Up changed route and opened its detail; Home/End selected Files/Archive. At 640, the wide outline, rows, splitter, and detail appeared together; the same sidebar keys selected routes, and Tab from Select Files moved visible focus to the note field. Return on Archive kept the selected destination available to reopen.

Focus did not visibly return to the Sidebar row after Back; Down had no effect until that row was clicked. This matches the documented GPUI focus boundary and establishes no source defect. AX reported the root window as focused rather than the focused child. VoiceOver was not run. These are requested widths only; actual rendered logical bounds were not independently measured. No source change was made.

Evidence: `target/headstart/evidence/navigation-split-keyboard-focus-native-probe-20261021.json` records the pre-fix observation. The locked/offline Playground build passed; its log is `target/headstart/evidence/m481-navigation-split-keyboard-build-20261021.log`. The validation app was stopped, and its disposable bundle and standalone executable were removed.


### M481 focus-restoration follow-up

The pre-fix M481 observation is preserved above and in its original evidence file. Back now focuses the retained Sidebar tree container through GPUI's window-aware action path; selection remains caller-owned. The focused regression passed 1/1. In the native follow-up at a requested 560 px, Back returned to Files and an immediate Down, with no row click, selected Settings and opened its detail. CUA AX reported the root window as focused, and the screenshot did not distinguish a visible tree ring from pointer hover, so visible focus is unconfirmed. VoiceOver was not run.

Evidence: `target/headstart/evidence/navigation-split-focus-restore-native-followup-20261021.json`; focused test log: `target/headstart/evidence/m481-navigation-split-focus-restore-test-20261021.log`.
