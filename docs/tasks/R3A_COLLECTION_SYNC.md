# R3a: collection selection synchronization

Scope: List, Table, and Tree owner selection updates. This is the independent selection synchronization portion of R3; R2 implements the approved mixed retention policy. M75/M76 close the bounded native Tree/Table interaction checks documented below. Broader collection identity/reorder contracts and complete GUI acceptance remain open.

## Pinned upstream behavior

- `ListState::set_selected_index` sets its index and calls the delegate setter without emitting a `ListEvent` (`vendor/gpui-component-0.7.3/src/list/list.rs:184`). User select, confirm, and cancel paths emit `ListEvent` (`list/list.rs:364`, `:385`, `:397`). List maps visible rows back to source indices before writing the owner.
- `TableState::set_selected_row` emits `SelectRow` and `clear_selection` emits `ClearSelection`, including when RustUI calls these setters to apply an external binding (`vendor/gpui-component-0.7.3/src/table/state.rs:502`, `:624`). `Context::emit` queues an effect, and GPUI flushes effects at the end of an app update (`vendor/gpui-pre-0.3.9/src/app/context.rs:780`, `src/app.rs:1997`). A short-lived guard around a setter cannot reliably suppress its event at delivery.
- `TreeState::set_items` and `set_selected_item` call `cx.notify()` (`vendor/gpui-base-0.7.3/src/tree.rs:221`, `:237`). RustUI observes notifications and compares the selected stable ID. These notifications can follow an external selection or data change.

## Change

For `List`, Escape clears the controlled selection only when `.selectable(true)`; with `.selectable(false)`, Escape preserves both the owner value and pending selection request.

Each collection event or observer now uses `RenderContext::binding_updater`, which reads the current owner value through `Binding::read_from` and writes only when the requested selection differs. This prevents redundant owner writes and host notifications from repeated user events and deferred upstream setter events. List and Table retain source-index selection; Tree retains stable-ID selection.

Table additionally retains the raw owner selection last seen at render or accepted from a UI event. A queued event is ignored if the owner value differs from that reconciliation marker, or if its payload no longer matches `TableState::selected_row()`. Accepted events advance the marker so multiple queued UI events can compose. **Selected policy: render-observed precedence.** An owner value observed during reconciliation takes precedence over an older queued event. An external A→B→A change completed before reconciliation is indistinguishable from no change; in that narrow ordering case, an already queued native event can still update the owner. This is source-derived and has not been reproduced as a runtime defect. The marker uses the raw value; an out-of-range owner index is normalized through the existing clear path.

List and Table selection are positional: reorder keeps a valid numeric source index, so it may identify a different row after reorder. Filtering a List row out of view retains its source-index selection. Removing enough List items to make the selected source index out of range now clears selection; Table also clears selection only when the selected index becomes out of range, while a still-valid index continues to select that slot's new occupant. Tree selection follows the selected stable ID through reorder or replacement while that ID remains present, and clears when the selected ID is removed. Tree expansion for unchanged node IDs follows the retained state when the expanded declaration is unchanged; changed declarations take effect, and removed IDs no longer retain expansion state. `KeyedList` selection follows its stable ID through insertion, reorder, and filtering, and clears if the selected ID is removed (`docs/tasks/R3B_KEYED_LIST.md`).

## Boundaries and follow-up interaction checks

- The change does not alter fallback identities, selection pruning, or Tree expansion policy. Those depend on R2.
- Tree retention across arbitrary root-topology changes requires a stable explicit `.id(...)`. Without one, `TreeView::identity()` derives its fallback from root count and first root ID; changing either remounts the retained `TreeState`. This is the documented boundary, not a source identity change: an anonymous Tree cannot infer an identity that survives arbitrary topology changes while distinguishing multiple Trees in one scope.
- The owner API exposes a value, not a change generation. Contract A intentionally accepts that a value changed away and back before reconciliation is indistinguishable from an unchanged value. Contract B would require mutation provenance and event sequencing not supplied by the current binding and Table event APIs; it is deferred as a future option in `docs/tasks/R3B_SELECTION_PROVENANCE_PROPOSAL.md`.
- The M75/M76 checks below cover Tree replacement/selection/expansion and Table invalid-index normalization/reorder/removal/native selection. They do not instrument owner-notification counts or establish ABA/away-and-back provenance behavior.

## M75/M76 bounded native closeout

Native CUA observations used binary SHA-256 `f7d78d68b7c688f4d5ca7ab24148df751b913cada32038df15d547dc05853b53` and are recorded in `target/headstart/milestone75-tree-gui.json` and `target/headstart/milestone76-table-gui.json`.

- **Tree (M75):** After collapsing a nested child, replacing/reordering/relabeling while the same ancestor remained collapsed preserved the child's collapsed state. Explicit `Workspace` expansion changes applied. Removing and restoring a branch reset its nested child to expanded. Selection followed `Grandchild` by stable ID through reorder; removing the selected ID cleared the binding. When a descendant remained selected, a collapsed ancestor declaration did not hide it because GPUI auto-expanded the ancestors.
- **Table (M76):** Invalid source indices normalized to `None`, both from no selection and from a valid selection. Reordering preserved the numeric source index, so it selected the row now at that index. Removing a row before the selected index could make that index select a different row; removing the last selected index cleared selection. Native row clicks selected the expected row; repeat click preserved `Some(1)` and explicit clear produced `None`.
- The combined workspace run passed 248 tests across 12 binaries (`target/headstart/milestone75-76-workspace-tests.log`). The final Headstart build exited 0; GPUI and GPUI Components artifacts were fresh (`target/headstart/milestone75-76-final-build.jsonl`, `.log`).
- Limits: no exact-pixel, full keyboard, screen-reader/AX, RTL, cross-platform, or performance acceptance. No Table owner-notification counter was instrumented. These observations do not claim an ABA/provenance fix. Prior failure records remain available in `milestone75-tree-selection-before.json` and `milestone76-table-invalid-before.json`.

## M80 bounded Collection native acceptance

The patched Playground build (`target/headstart/milestone80-collection-final-playground-build.log`) passed the focused native replay. From the initial first rows, **Select last row** selected `record-0999` and moved the visible viewport to `record-0994`–`record-0999`. The replay edited row `record-0000` to `durable row zero`, incremented rows `record-0000` and `record-0001`, scrolled to the end and back, and confirmed all three values remained. Clearing selection and selecting the last row again from the top also moved to the final rows. Final evidence: `target/headstart/milestone80-collection-native-final.json`; source binary SHA-256 is recorded there.

Headstart validation: 8 focused Collection tests passed (`target/headstart/milestone80-collection-final-tests.log`), `rustui --all-targets` check passed (`target/headstart/milestone80-collection-final-check.log`), and the Playground build passed. The app was quit and its disposable bundle and standalone executable removed; Cargo artifacts were retained. This bounded run does not establish reorder/removal/restore behavior, keyboard-only focus, full lifecycle cleanup, accessibility/screen-reader, cross-platform, or performance acceptance.

## M82 - mutation and restore acceptance

The Playground catalog search now resets its virtualized viewport on query changes, so a matching story remains visible regardless of the previous catalog scroll offset. The Collection story stores the removed row's stable previous/next IDs, original index, and order direction. Restore uses the next neighbor in the same direction or previous neighbor after a reversal, falling back to the surviving alternate neighbor and then the clamped original index.

Native run: searched from the scrolled catalog for collection, opened the single result, changed record-0000 note to preserved row, incremented its counter to 1, selected and removed it, reversed remaining rows, restored it, then selected the final row. Observed Rows: 999, selected ID: None after removal; after restore, Rows: 1000, selected ID: record-0000; restored row showed count 1 and note preserved row. Lifecycle observation moved from active 6/mounts 6/releases 0 to active 6/mounts 7/releases 1 on removal and active 6/mounts 13/releases 7 after reversal/restoration. Focused tests: 3 passed. Playground all-targets check and build succeeded. Binary SHA-256: f44b6120476b8d0ed6df360dd6aa46a9bb42ce62690cb38255b3e8aa3eb6dbb7.

Evidence: target/headstart/milestone82-collection-mutation-gui.json, target/headstart/milestone82-collection-tests.log, target/headstart/milestone82-collection-check.log, and target/headstart/milestone82-collection-playground-build.log. Keyboard-only, screen-reader/accessibility, cross-platform, and performance acceptance remain open.


## M126 native source-List Escape check

With the source-index List's row 2 selected and `.selectable(false)`, clicking the selected East row and pressing Escape kept the owner selection at `Some(1)` and retained East as selected in the native tree. This is one bounded keyboard case; full keyboard traversal, screen-reader, RTL, cross-platform and performance acceptance remain open. Evidence: `target/headstart/evidence/r3-list-datagrid-acceptance-20261019.json`. Related M126 DataGrid native observation in the same evidence: the Quarterly revenue table exposed native row nodes, and selecting its first rendered row changed owner selection from None to Some(0); this does not establish full screen-reader or upstream ColumnHeader/Cell roles.

## M150 native source-List keyboard follow-up

On macOS 27, clicking `Text` then pressing Down changed the source-index owner to `Some(2)` and the native tree marked `TextField` selected. Home and End left `Some(2)` and `TextField` selected; no Home/End navigation support is claimed. Escape then changed the owner to `None` and the native tree showed all seven rows as selectable with none selected. CUA continued to report the root window as focused, so exact child-focus ownership was not established. Screen-reader speech, RTL, cross-platform, visual pixel comparison, and performance were not tested. This is bounded native evidence, not full keyboard acceptance. Evidence: `target/headstart/evidence/r3-source-list-keyboard-m150-20261020.json`.

## M155 Tree Enter confirmation source check

RustUI now binds Enter to GPUI Components' existing `Confirm` action only in
the `Tree` key context. The upstream handler toggles expansion for a selected
folder; selecting a leaf and pressing Enter has no activation effect. RustUI's
public `Tree` API still has no leaf-activation callback. The focused source
test verifies that the binding matches Enter and not Space. Output:
`target/headstart/evidence/r3-tree-enter-m155-tests.log`. This is a source check
only; native keyboard acceptance is still open.

## M156 native Tree Enter acceptance

On macOS 27 in the canonical Playground Tree story, selecting `Archive` and pressing Enter collapsed it and hid `Archived item`. Return collapsed it as well; Right expanded it again. This confirms the Enter mapping against the existing Tree expansion controls. It does not add leaf activation. Screen-reader speech, cross-platform behavior, RTL, and broader accessibility acceptance remain open. The Playground process was absent after close; its disposable app bundle was removed, with Cargo artifacts and caches preserved. Evidence: `target/headstart/evidence/r3-tree-enter-m156-20261020.json` (Playground executable SHA-256 `4ad52d366d00f48b8782c402cda6aba4ce2082d2fcb768fac352802529e0f378`).

## M183 List source-index normalization

List now clears owner selection when its source index becomes out of range after item removal. A valid source index hidden by search filtering remains selected. This matches Table's positional-selection normalization and retains a guard against clearing a newer owner selection. All six focused List unit tests pass; the locked/offline RustUI all-targets check and rustfmt check pass. No GUI acceptance was run. Logs: `target/headstart/evidence/r3-list-invalid-selection-m183-tests-20261020.log` and `target/headstart/evidence/r3-list-invalid-selection-m183-check-20261020.log`.

## Native source-List focus probe (2026-10-20)

Using the canonical Playground `Framework Demos > List` story, clicking `Text`
changed the owner to `Some(1)` and marked `Text` selected in the native AX tree.
Pressing Down changed the owner to `Some(2)` and marked `TextField` selected.
The AX snapshot reported the root standard window as the focused UI element both
after the click and after Down; exact List/row focus ownership therefore remains
unknown. VoiceOver speech/navigation was not tested. This bounded probe does not
close the broader keyboard or screen-reader gate. Evidence:
`target/headstart/evidence/r3-source-list-focus-probe-20261020.json` (rerun
against the current Playground executable, SHA-256
`217a85d8222e0932a81f2d7b41068f14b8c9f11fbf499154ac47e91c8c9ce1a4`).
# M408 — refresh retained List selection writers
List and KeyedList now refresh their retained selection writer on each render and dispatch events through the latest writer. This preserves writes when owner rerenders create a new `Binding::new_with` context. List tests passed 10/10 and KeyedList tests passed 11/11. Native interaction and accessibility acceptance remain open. Evidence: `target/headstart/evidence/stage8-list-latest-writer-m408-20261021.log`.


## M475 current-source native List acceptance (2026-10-21)
Rebuilt the current Playground source and opened the canonical `Framework Demos > List` story. Its explicit List identity remains `story-list-components` across owner updates. Clicking `Text` produced owner `Some(1)` with `Text` selected in the native AX tree; clicking `TextField` after that rerender produced `Some(2)` with `TextField` selected. Searching `Button` preserved owner `Some(2)` while the selected row was hidden; clearing search restored the selected `TextField` row. Every AX response reported the root window focused, so exact child focus ownership is unverified. VoiceOver speech/navigation was not tested. M478 later added the final-row removal fixture and recorded the M183 native GUI-update-cycle normalization case; see the M478 acceptance record below. M183's bounded normalization contract is covered. The broader R3 keyboard, screen-reader, RTL, platform, and performance acceptance remains open. Build and native evidence: `target/headstart/evidence/m475-playground-build-20261021.log`, `target/headstart/evidence/m475-source-list-native-acceptance-20261021.json`.


## M477 current-source native Table acceptance (2026-10-21)
Rebuilt current Playground source and opened `Framework Demos > Table`, whose controlled table keeps the explicit identity `table-reconciliation-rows`. Clicking native `Delta` at source index 3 reported owner `Some(3)` and highlighted the row. Removing that last row left Alpha/Beta/Gamma, cleared owner selection to `None`, and left no selected row. Clicking native `Alpha` reported `Some(0)`; reversing the rows kept `Some(0)` and highlighted `Gamma` at its new position. Setting an invalid index normalized the owner to `None`; restoring the rows kept selection empty. AX exposed the named Table with row/cell hierarchy but reported the root window focused, so exact child focus ownership remains unverified. The fixture has no filter control. VoiceOver speech/navigation was not tested, and broader R3 acceptance remains open. Evidence: `target/headstart/evidence/m477-playground-build-20261021.log`, `target/headstart/evidence/m477-table-native-acceptance-20261021.json`.

## M476 current-source native Tree acceptance (2026-10-21)

Sizing the intermediate RustUI action-capture wrapper restored the 280-point reconciliation Tree rows. On current-source macOS 27 Playground, `Workspace`, `Branch`, nested folders, `Grandchild leaf`, `Stable sibling`, `Archive`, and `Archived item` were visible in both themes and exposed in the native accessibility tree. Selecting `Workspace` updated the controlled stable ID; reversing roots preserved that selection. Toggling the declared expansion collapsed and restored descendants while preserving selection. Selecting `Grandchild leaf` updated selection and recorded one activation. Removing its branch cleared selection to `None`; restoring the branch returned its declared expanded state. Exact child focus is unverified; VoiceOver was not run. RTL, cross-platform, performance, and broader R3 acceptance remain open. Evidence: `target/headstart/evidence/m476-tree-native-acceptance-20261021.json`.

## M478 current-source native List removal acceptance (2026-10-21)

The canonical List story now owns mutable rows in a same-identity reconciliation specimen with a final-row removal control. Selecting `ColorPicker` at source index 6 reported owner `Some(6)`; removing it reduced the row count to six, removed its native AX row, and normalized selection to `None`. Selecting `TextField` reported `Some(2)`; a `Button` filter hid it without clearing that valid selection, and clearing the filter restored the selected row. The List identity remained `story-list-components`. Exact child focus is unverified; VoiceOver was not run. RTL, cross-platform, performance, and broader R3 acceptance remain open. Evidence: `target/headstart/evidence/m478-list-removal-native-acceptance-20261021.json` and `target/headstart/evidence/r3-m476-m183-playground-build-20261021.log`.


## M479 current-source native List restoration acceptance (2026-10-21)

The existing `ListReconciliationGallery` now restores its initial rows through the same List identity, `story-list-components`, while leaving owner selection unchanged. On macOS 27, selecting final row `ColorPicker` reported `Some(6)`; removing it reduced rows from seven to six and normalized selection to `None`; restoring rows returned all seven items, including `ColorPicker`, while selection stayed `None`. Build passed with `./scripts/cargo-headstart build -p rustui-playground --locked --offline`. Evidence: `target/headstart/evidence/m479-list-restore-native-acceptance-20261021.json`. Child focus, VoiceOver, RTL, cross-platform, performance, and broader R3 acceptance remain open.

### M475 child-focus and VoiceOver follow-up (2026-10-21)

Rebuilt and opened the existing disposable validation identity (`dev.rustui.playground.validation`) on macOS 27. In `Framework Demos > List`, selecting the `Button` row changed the selected source row to `Some(0)` and marked the row container selected in the AX tree. After Tab navigation, the AX state continued to report the standard window as the focused UI element. VoiceOver was enabled for a brief follow-up and restored to off; CUA did not expose a spoken transcript, so child focus and spoken role/name/selected-state acceptance remain unverified. This probe did not establish a source defect, so no List implementation change was made. The app was quit and the disposable bundle removed; Cargo build artifacts were preserved.

Evidence: `target/headstart/evidence/r3a-list-child-focus-voiceover-attempt-20261021.json` and `target/headstart/evidence/r3-list-child-focus-playground-build-20261021.log`.
