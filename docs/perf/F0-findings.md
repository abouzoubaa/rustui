# F0 Findings

## Button disabled accessibility — resolved

- Test: `components::disabled_tests::button::accessibility`.
- Cause: GPUI Fast's `window/a11y/debug.rs::node_to_json` omitted `node.is_disabled()`, so the test tree did not serialize disabled state even when the AccessKit node held it.
- Evidence: before the serializer patch, the fresh-target tree printed the Target Button without a `disabled` field. After the patch, a fresh build printed the direct disabled node as `{"role":"Button","disabled":true}`; both the start-disabled diagnostic and `components::disabled_tests::button::accessibility` passed.
- Patch: `gpui-fast-test-tree-disabled-v1-fast` serializes the node disabled state, bounds, and `labelled_by` IDs, with the vendored checksum entry updated. It is applied by `scripts/apply_f0_fast_candidate_patch.py`; reverse/apply/check and candidate stack checks pass.
- Fresh-target proof: `components::disabled_tests` passed 201/0/0, and `grid_cells_reference_actual_header_nodes_across_updates` passed 1/0/0. The bounds serialization also fixed popup interaction targeting in the disabled matrix. No assertion was changed or ignored.

## Remaining full-suite failure

- Test: `components::advanced::graph_view::interaction::current_selection_regressions::rendered_node_editor_connects_moves_and_deletes_nodes`.
- Evidence: fresh-target `scripts/check.sh` on this branch reports 1,435 passed, 1 failed, 7 ignored. The failure is at `interaction.rs:1450`: actual node position `(0.0, 0.0)`, expected `(40.0, 25.0)` after a graph reset and simulated drag.
- Cause: still under investigation. An attempted `GPUI_VIEW_RETENTION=0` diagnostic stopped earlier in GPUI Fast glyph bounds with a NaN/Inf debug assertion, so it did not isolate this failure. No GPUI patch has been tried for this case, and the assertion remains unchanged.
