# F0 Findings

## Button disabled accessibility

- Test: `components::disabled_tests::button::accessibility`.
- Cause observed: the RustUI Button's accessibility node does not receive the disabled state from the Kit Button render path. The test tree contains one Button node for `Target`, and that node has no `aria.disabled` field. Starting disabled before the window's first frame fails too, so the current evidence does not indicate a stale-state-only failure.
- Evidence: with a fresh `CARGO_TARGET_DIR=$HOME/.cache/rustui-target-f0diag` and Rust 1.98.0, the test fails at `disabled_tests.rs:938`: `Button: disabled AX node: {"role":"Button","on_action":["Click"],"label":"Target"}`. The start-disabled diagnostic printed interactive nodes `a` Before Button, `b` Target Button, `c` R15 region Group, `d` After Button, and `e` Window; none exposed `aria.disabled`.
- Patch tried: `gpui-component-button-disabled-a11y-v1-fast`, applied and reversed through `scripts/apply_f0_fast_candidate_patch.py`; adding `.aria_disabled(disabled)` to the Kit Button render chain did not change the tree. It was removed from the active candidate stack pending a correct node-level diagnosis.
- Status: finding for F0-5 RustUI/Kit accessibility follow-up; assertion retained, no ignore added.
