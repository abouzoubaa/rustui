# M1194 — Stage 3 interactive Menu Bar sidebar

## Scope

Give the illustrative Playground Menu Bar composition a stateful View menu
action without adding a native menu-bar renderer.

## Change

The View menu toggles a visible Favorites/Downloads sidebar, changes its action
label between show and hide, and reports the resulting state.

## Verification

The focused source regression dispatches the component's generated `Action`
for the View menu toggle and each File/Edit/View/Item/Window/Help menu action.
It checks owner notification for every dispatch, the toggle's label and status,
and the `Favorites`/`Desktop`/`Downloads` sidebar projection. It does not
activate the rendered Menu event handler.

Focused check passed:

```sh
./scripts/cargo-headstart test -p rustui-playground --lib menu_bar_composes_public_menu_controls_and_dispatches_owner_actions --locked --offline
```

Result: 1 passed, 0 failed, 182 filtered out. Visual, keyboard, and
accessibility acceptance for the illustrative composition remain open; a
native menu-bar renderer is outside this task's scope.

## 2026-10-24 focused recheck

The same regression passed again in the current checkout:

```sh
./scripts/cargo-headstart test -p rustui-playground --lib --locked --offline menu_bar_composes_public_menu_controls_and_dispatches_owner_actions --quiet
```

Result: 1 passed, 0 failed, 182 filtered out. The check covers generated
component-action dispatch and its notification result, not the rendered Menu
event path. Native visual, keyboard, and accessibility acceptance were not
rerun. The existing Playground and validation app processes were left untouched.

## 2026-10-24 menu action dispatch follow-up

The regression now also dispatches File, Edit, Item, Window, and Help through
the generated owner `Action`, checking notification and resulting status for
each action. View's toggle remains covered in both directions. The focused
locked/offline check passed 1/1 with 182 tests filtered out:

```sh
./scripts/cargo-headstart test -p rustui-playground --lib menu_bar_composes_public_menu_controls_and_dispatches_owner_actions --locked --offline
```

This still does not activate the rendered Menu event handler or establish
native visual, keyboard, or accessibility acceptance.

## 2026-10-24 Playground library regression recheck

The full Playground library suite passed after refreshing stale catalog-note
expectations and the compiled-preview color-channel rejection fixture:

```sh
./scripts/cargo-headstart test -p rustui-playground --lib --locked --offline --quiet
```

Result: 183 passed, 0 failed. This source-level suite does not close the
rendered Menu event-path or native visual, keyboard, and accessibility gates.

A separate in-process popup simulation was not added: enabling the pinned
GPUI `test-support` feature requires `futures-concurrency`, which is absent
from the workspace's vendored source and cannot be resolved by the locked,
offline test command. No test-support feature or vendor change was retained.

## 2026-10-24 accessibility source follow-up

The File/Edit/View/Item/Window/Help and sidebar action feedback now uses the
existing polite Status live-region semantics. The focused owner-action test
was not rerun after this source change because the current Headstart wrapper
fails before compiling the Playground crate with `E0463: can't find crate for
rustui`; see the M1205 follow-up. Rendered Menu activation and native visual,
keyboard, and accessibility acceptance remain open.

## 2026-10-24 isolated focused retry

The documented owner-action test was retried with a persistent isolated target:

```sh
CARGO_TARGET_DIR=/Users/Abium/Developer/rustui/target/headstart/m1194-live-region-owner-action ./scripts/cargo-headstart test -p rustui-playground --locked --offline --lib menu_bar_composes_public_menu_controls_and_dispatches_owner_actions
```

Cargo exited 101 during offline dependency resolution because the vendored
source had no matching `futures-concurrency` package. The Playground crate did
not compile and the test assertions did not run. No source or dependency
changes were made. The rendered Menu event path and native visual, keyboard,
and accessibility acceptance remain open.

## 2026-10-24 focused existing-target execution

The exact owner-action test was then run from the persistent M1205 test target.
The Menu Bar source timestamp (06:41 UTC) predates the test executable
timestamp (22:15 UTC), so the executable includes the current story source.

```sh
target/headstart/m1205-live-region-initial-status/debug/build/rustui-playground/674e94dd222875bd/out/rustui_playground-674e94dd222875bd stories::menu_bar_reference_gallery::tests::menu_bar_composes_public_menu_controls_and_dispatches_owner_actions --exact
```

Result: 1 passed, 0 failed; 182 filtered. This reuses the previously compiled
test binary rather than rebuilding under the current Cargo lockfile state. It
covers generated owner-Action dispatch, not the rendered popup-row event path.
Native visual, keyboard, and accessibility acceptance remain open.

## 2026-10-24 Menu callback source trace

`MenuView::into_element` binds item callbacks before constructing the dropdown.
`bind_items` wraps each item's `Action` with `context.bind_control_action`,
`append_menu_items` passes that callback to the popup row, and
`popup_action_item` invokes it from the row's click handler when enabled. This
confirms the RustUI source callback path only. The owner-action regression
dispatches generated `Action` values directly; it does not activate the
rendered popup row. Rendered Menu activation and native acceptance remain open.

## 2026-10-24 native acceptance preflight

Native acceptance was not run because three Playground processes were
observed (PIDs 10507, 23568, and 36564) and their owners were unknown. They
were left untouched. Retry only after the owners are identified and the shared
slot is available; first resolve the recorded `E0463` and rerun the focused
owner-action regression, then follow the run sheet.

## Native Playground acceptance run sheet

Run only when the shared Playground slot is available and the owner of any
currently running Playground process is known. Preserve unowned processes. Use
the canonical Playground and its existing disposable validation app bundle
identifier; verify that the launched build contains the current
`menu_bar_reference_gallery.rs` source, not the older served Web bundle.

1. Open the `menu_bar` catalog story. Record the build identity and capture the
   initial illustration: File, Edit, View, Item, Window, Help; Favorites,
   Desktop, Downloads; and the initial `Hide Sidebar` action.
2. Open the rendered View menu and activate `Hide Sidebar`. Pass when the
   sidebar disappears, the action changes to `Show Sidebar`, and the visible
   status reads `Sidebar hidden`. This must be the popup row's real click path,
   not direct generated-Action dispatch.
3. Activate `Show Sidebar`. Pass when Favorites, Desktop, and Downloads return,
   the action changes back to `Hide Sidebar`, and the status reads
   `Sidebar shown`.
4. Activate File, Edit, Item, Window, and Help actions from their rendered
   popup rows. Confirm each produces its corresponding `… menu action selected`
   status. Record any menu item that cannot be reached or activated.
5. Repeat the hide/show path using keyboard focus and the menu's supported
   keyboard navigation. Record the exact keys used. Pass when both actions
   update the same sidebar, label, and status state and the popup can be
   dismissed without losing the expected focus path.
6. Inspect the live accessibility tree and VoiceOver output while hiding,
   showing, and selecting the other actions. Pass when the menu controls have
   their visible names, the changing feedback is exposed as a polite Status
   live region, and the updates are announced. Save the tree observation and
   VoiceOver result separately.

Before the GUI run, rerun the focused locked/offline owner-action regression
after resolving the recorded `E0463: can't find crate for rustui` build failure.
That source check does not substitute for steps 2–6. Keep native menu-bar
rendering outside this task. After the GUI run, quit only the test app launched
for this run, remove its disposable bundle and standalone test executable,
preserve shared Cargo caches, and verify that no test app remains running.

### Required observation record

Record these expected-versus-observed results for the same build. Use
`pass`, `fail`, or `not run`; do not infer a pass from source dispatch.

| Action | Expected visible result | Expected status |
| --- | --- | --- |
| Initial story | File, Edit, View, Item, Window, Help; Favorites, Desktop, Downloads; `Hide Sidebar` | Initial story state |
| View → Hide Sidebar | Sidebar absent; action is `Show Sidebar` | `Sidebar hidden` |
| View → Show Sidebar | Favorites, Desktop, Downloads restored; action is `Hide Sidebar` | `Sidebar shown` |
| File | Sidebar unchanged | `File menu action selected` |
| Edit | Sidebar unchanged | `Edit menu action selected` |
| Item | Sidebar unchanged | `Item menu action selected` |
| Window | Sidebar unchanged | `Window menu action selected` |
| Help | Sidebar unchanged | `Help menu action selected` |

For each keyboard hide/show run, record the exact key sequence, focused
control before opening, selected menu item, resulting sidebar/action/status,
and focus after dismissal. For accessibility, record the trigger and each
popup item's role/name/focusability, then the status element's role, value,
and politeness after every action. Capture the literal VoiceOver announcement
for `Sidebar hidden`, `Sidebar shown`, and each menu action; mark an absent or
incorrect announcement as a failure.

Save the app bundle identifier and path, binary/source revision identity, run
timestamp, screenshots for initial/hidden/shown states, accessibility-tree
observations, VoiceOver notes, and a result for every row above with the
acceptance evidence. If any state or announcement differs from its expected
value, retain the failure observation and leave M1194 acceptance open.
