# F0 Findings

## Button disabled accessibility — resolved

- Test: `components::disabled_tests::button::accessibility`.
- Cause: GPUI Fast's `window/a11y/debug.rs::node_to_json` omitted `node.is_disabled()`, so the test tree did not serialize disabled state even when the AccessKit node held it.
- Evidence: before the serializer patch, the fresh-target tree printed the Target Button without a `disabled` field. After the patch, a fresh build printed the direct disabled node as `{"role":"Button","disabled":true}`; both the start-disabled diagnostic and `components::disabled_tests::button::accessibility` passed.
- Patch: `gpui-fast-test-tree-disabled-v1-fast` adds the `node.is_disabled()` JSON field and updates the vendored checksum entry. It is applied by `scripts/apply_f0_fast_candidate_patch.py`; core stack exact-hash check passes.
- The initial Kit Button `.aria_disabled` attempt was removed because it did not address the serializer omission. The original assertion remains unchanged; no test is ignored.
