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

Table selection is positional: reorder keeps a valid numeric source index, so it may identify a different row after reorder. Deleting rows clears selection only when the selected index becomes out of range; a still-valid index continues to select that slot's new occupant. Tree selection follows the selected stable ID through reorder or replacement while that ID remains present, and clears when the selected ID is removed. Tree expansion for unchanged node IDs follows the retained state when the expanded declaration is unchanged; changed declarations take effect, and removed IDs no longer retain expansion state. `KeyedList` selection follows its stable ID through insertion, reorder, and filtering, and clears if the selected ID is removed (`docs/tasks/R3B_KEYED_LIST.md`).

## Boundaries and follow-up interaction checks

- The change does not alter fallback identities, selection pruning, or Tree expansion policy. Those depend on R2.
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

With the source-index List's row 2 selected and `.selectable(false)`, clicking the selected East row and pressing Escape kept the owner selection at `Some(1)` and retained East as selected in the native tree. This is one bounded keyboard case; full keyboard traversal, screen-reader, RTL, cross-platform and performance acceptance remain open. Evidence: `target/headstart/evidence/r3-list-datagrid-acceptance-20261019.json`.
