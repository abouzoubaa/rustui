# M786 — Characterize the Web keyboard focus boundary

## Browser observations

In the running Playground browser bundle:

- Pointer activation changed the counter from 0 to 1 to 2.
- The packaged `Open Form.svg` link opened and rendered the SVG.
- Selecting Arabic changed the visible layout to RTL.
- The GPUI canvas had `tabindex=-1`. Explicit `canvas.focus()` made it `document.activeElement`; Space targeted the canvas but did not activate the selected Button.
- The page exposed a canvas and hidden IME textarea, with no usable per-control browser accessibility nodes observed.

## Boundary

Pinned GPUI Web setup makes the canvas non-tabbable and routes keyboard events through its hidden IME textarea. The observed keyboard gap is in that host event/focus path; no RustUI-owned workaround was verified. Changing the pinned host requires a separately approved, reproducible patch. Stage 11 keyboard and browser screen-reader acceptance remain open.


## Candidate review update — 2026-10-22

Read-only review of proposed host patch v1 found two blockers: its `handle_key_down(&self, ...)` calls `schedule_ime_mirror_sync(self: &Rc<Self>)`, which is a compile-time receiver mismatch; and the patch adds canvas `keydown` handling without a matching canvas `keyup` route, while key-up remains attached to the hidden IME textarea. The proposal checker now rejects both conditions. `python3 -m py_compile` passed for the checker; running it rejects v1 on those invariants. No Rust compilation or new browser regression run was performed, and the proposed patch was not applied or changed. A corrected patch still needs the required approval before application.

## Candidate update — 2026-10-22 (v2)

The v2 proposal fixes the keydown helper receiver and adds canvas keyup routing while retaining both hidden IME input listeners. python3 scripts/check_gpui_web_keyboard_focus_activation_patch.py passed: the exact patch applied to disposable source copies, source and patch hashes and keyboard path invariants matched, and scoped rustfmt passed. Vendored files were not changed. This does not compile GPUI or verify browser input/accessibility behavior. Explicit review and approval are still required before applying the patch, followed by browser acceptance. See docs/tasks/M855_STAGE11_WEB_KEYBOARD_FOCUS_PATCH_V2.md.
