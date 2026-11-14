# F0 Findings

## Button disabled accessibility — resolved

- Test: `components::disabled_tests::button::accessibility`.
- Cause: GPUI Fast's `window/a11y/debug.rs::node_to_json` omitted `node.is_disabled()`, so the test tree did not serialize disabled state even when the AccessKit node held it.
- Evidence: before the serializer patch, the fresh-target tree printed the Target Button without a `disabled` field. After the patch, a fresh build printed the direct disabled node as `{"role":"Button","disabled":true}`; both the start-disabled diagnostic and `components::disabled_tests::button::accessibility` passed.
- Patch: `gpui-fast-test-tree-disabled-v1-fast` serializes the node disabled state, bounds, and `labelled_by` IDs, with the vendored checksum entry updated. It is applied by `scripts/apply_f0_fast_candidate_patch.py`; reverse/apply/check and candidate stack checks pass.
- Fresh-target proof: `components::disabled_tests` passed 201/0/0, and `grid_cells_reference_actual_header_nodes_across_updates` passed 1/0/0. The bounds serialization also fixed popup interaction targeting in the disabled matrix. No assertion was changed or ignored.

## GraphView regression — resolved

- Test: `components::advanced::graph_view::interaction::current_selection_regressions::rendered_node_editor_connects_moves_and_deletes_nodes`.
- Cause: the test reset its graph owner without `cx.notify()`, leaving Fast's retained view at the previous layout. When the test reset now notifies and redraws, GPUI Fast's `LineGlyphPainter::may_reach` debug assertion compares identical bounds with a NaN width using ordinary float equality; NaN compares unequal to itself.
- Fix: the test setup now notifies after restoring the graph, preserving its drag assertions. Candidate patch `gpui-fast-glyph-mask-nan-equality-v1-fast` treats matching NaN coordinates as equal in that debug-only consistency check. The focused test passed on a fresh target with the patch compiled. The test assertion was not changed or ignored.
- Before the fix, full `scripts/check.sh` was 1,435 passed, 1 failed, 7 ignored. After the fix, full `scripts/check.sh` passed with 1,436 passed, 0 failed, 7 ignored (Rust 1.98.0).
