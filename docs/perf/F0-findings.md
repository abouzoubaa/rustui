# F0 Findings

## F0-4a: Fast accessibility and draw assertions

- Full-check baseline after F0-4a: **1,388 passed / 48 failed / 7 ignored**. The same totals reproduce with `CARGO_TARGET_DIR=$HOME/.cache/rustui-target-f0diag scripts/check.sh` on Rust 1.98.0; the isolated build rules out stale workspace artifacts.
- `components::disabled_tests::button::accessibility` fails both after enabled→disabled and with the owner disabled before its first rendered frame. The latter dump has one Target node: `{"role":"Button","on_action":["Click"],"label":"Target"}`; no node has `disabled`. The full-tree dump is in the command output from the temporary diagnostic test, which was removed. The `gpui-fast-test-tree-disabled-v1-fast` candidate serializes `node.is_disabled()` when present, but the target node reaching that serializer has no disabled state. No new patch was tried because start-disabled did not pass.
- `components::data::editable_data_grid::tests::grid_cells_reference_actual_header_nodes_across_updates` fails: its Cell node has row index 2 and column index 1 but no `labelled_by`; the assertion says the relation must target the actual header NodeId. The candidate serializer exposes `labelled_by`, but the emitted Cell node has no relation to serialize.
- `components::advanced::graph_view::interaction::current_selection_regressions::rendered_node_editor_connects_moves_and_deletes_nodes` still fails in the full isolated-target run inside `LineGlyphPainter::may_reach` with identical displayed bounds containing `NaNpx`. The `equal_including_nan` candidate is present and compiled; its assertion still fires. Root cause remains unresolved.
- No test assertions were weakened, removed, or ignored.

## F0-4b: Menu/select family

- Copied 10 menu/select patches unchanged from `patches/approved/kit09/` into `patches/candidate-kit-fast/`; `menu-select-stack.json` records provider-first order. `test_menu_select_candidates_are_exact_kit09_copies` passes.
- Cumulative Kit 0.9 fixture stack applied and reversed exactly (64 stack files). The disabled popup-menu confirmation proof and popup-menu selection-recovery proof each passed: 1 passed / 0 failed / 0 ignored. This fixture uses the Kit 0.9.1 source archives; the workspace Fast compilation also passes, but RustUI's disabled/popup assertions below still fail.
- Full RustUI check after F0-4b: **1,388 passed / 48 failed / 7 ignored**. Menu/select failures: `components::disabled_tests::menu_item::accessibility` finds no MenuItem AX node; `menu_item::recovery` fails to find the popup row; `combo_box::recovery` and `picker::recovery` unwrap a missing popup node. The ComboBox/Picker accessibility cases also report `disabled: null`, matching the F0-4a disabled-state finding.
- No assertions were weakened, removed, or ignored. Continue these findings into F0-5/F0-7 analysis.

## F0-4c: Tree/list/grid family

- Copied 8 tree/list/grid patches unchanged from `patches/approved/kit09/`. The candidate copy/hash test passes. The cumulative Kit 0.9 fixture applies and reverses 64 stack files exactly; focused tree-name/level, DataTable delegate-accessibility, and ARIA sort/read-only proofs each pass (1 passed / 0 failed / 0 ignored).
- Full RustUI Fast check remains **1,388 passed / 48 failed / 7 ignored**. `components::data::editable_data_grid::tests::grid_cells_reference_actual_header_nodes_across_updates` still fails because the emitted Cell node has no `labelled_by` relation to its header. Tree disabled accessibility and recovery cases remain among the disabled-matrix failures; see F0-4a.
- No assertions were weakened, removed, or ignored.

## F0-4d: Slider and resize family

- Copied six slider/resize patches unchanged from the Kit 0.9 candidate stack. The exact-copy test and core candidate stack checks pass. The cumulative Kit 0.9.1 fixture applied and reversed all 64 stack files exactly.
- Focused fixture proofs pass: slider SetValue/Increment/Decrement; tested value snapping; endpoint and thumb geometry; resize first pointer position; resize release handling (each 1 passed / 0 failed / 0 ignored).
- Full RustUI Fast check remains **1,388 passed / 48 failed / 7 ignored** (exit 101). The failure set remains 47 accessibility/recovery cases plus the Grid header relation and GraphView NaN assertion; none is a slider/resize assertion. No test was weakened, deleted, or ignored.

## F0-4e: Calendar and DatePicker family

- Copied eight calendar/DatePicker patches unchanged from the Kit 0.9 candidate stack. The candidate-copy/hash unit suite passes 5/5, including this family. The cumulative Kit 0.9.1 fixture applied and reversed all 64 stack files exactly.
- Focused fixture proofs pass: French/Arabic calendar translations; uncommitted calendar keyboard cursor; DatePicker confirm behavior; RTL calendar and DatePicker accessibility; date limits and multi-month count (each 1 passed / 0 failed / 0 ignored).
- Full RustUI Fast check remains **1,388 passed / 48 failed / 7 ignored** (exit 101), with the same 47 disabled-control accessibility/recovery failures, Grid header relation failure, and GraphView NaN assertion. No calendar/DatePicker behavior assertion failed. No test was weakened, deleted, or ignored.

## F0-4f: Input/text family

- Copied two input/text patches unchanged from the Kit 0.9 candidate stack. The candidate-copy/hash unit suite passes 6/6. The cumulative Kit 0.9.1 fixture applied and reversed all 64 stack files exactly.
- Focused fixture proofs pass: placeholder color override survives palette resolution; validation description reaches the editor-owned accessibility node (each 1 passed / 0 failed / 0 ignored).
- Full RustUI Fast check remains **1,388 passed / 48 failed / 7 ignored** (exit 101). The failing tests remain 47 disabled-control accessibility/recovery cases, the Grid header relation, and GraphView's NaN assertion. No input/text behavior assertion failed. No test was weakened, deleted, or ignored.
