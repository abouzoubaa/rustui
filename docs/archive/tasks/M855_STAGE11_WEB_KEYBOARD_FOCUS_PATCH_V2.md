# M855 — Stage 11 Web keyboard focus and activation patch candidate

The v2 proposal keeps keyboard listeners on the hidden IME input and adds matching canvas `keydown` and `keyup` routes. Its shared keydown handler uses an `&Rc<Self>` receiver for IME mirror scheduling. The checker applies the proposal only to disposable copies, validates the pinned source and patch hashes, checks both event paths and key event dispatch, and runs `rustfmt` on the disposable result.

The proposal changes no vendored files. It remains unapproved and unapplied. Passing the isolated checker does not prove that GPUI compiles or that keyboard activation, text entry, screen-reader behavior, and browser accessibility work at runtime. Review and explicit approval are required before applying the GPUI patch; browser acceptance remains open afterward.

Evidence: `scripts/check_gpui_web_keyboard_focus_activation_patch.py`, `patches/proposed/gpui-pre-web-keyboard-focus-activation-v2.patch`.

Focused checker rerun: `python3 scripts/check_gpui_web_keyboard_focus_activation_patch.py`
passed. It verified the proposal hash, exact application to disposable source
copies, patched source hashes, keyboard-path invariants, and `rustfmt`; pinned
vendor files were not modified. This does not replace GPUI compilation or
browser input/accessibility acceptance.

## Correction — current checker target (2026-10-23)

The checker now targets v3, not the v2 patch named in this historical note.
Use `docs/tasks/M855_STAGE11_WEB_KEYBOARD_FOCUS_PATCH_V3.md` for the current
proposal and its evidence. V2 remains unapplied and is not validated by the
current checker.
