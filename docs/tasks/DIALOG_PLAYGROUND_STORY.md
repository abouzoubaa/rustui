# Dialog Playground story

## Scope

Add a dedicated RustUI Playground demonstration for the public trigger-driven `Dialog` API. The story opens one dialog and displays the result of its Confirm or Cancel action.

## Acceptance

- The existing `save_dialog` catalog entry opens the dedicated story.
- The story uses only public `rustui::prelude` APIs.
- Confirm and Cancel each produce visible result text.
- The demonstration describes the single-topmost dialog contract. It makes no claim about nesting, controlled visibility, owner-removal closure, or patch-backed behavior.
- Build and formatting checks pass. A bounded macOS interaction probe reached the `save_dialog` story and observed the accessibility-tree result text change to `Confirmed: action accepted.` after Return and `Cancelled: no action taken.` after Escape. The screenshot remained an unchanged Button story while the accessibility tree reflected Dialog state, so visible rendering, modal focus, accessibility semantics, theme appearance, and locale behavior remain unverified. Exact steps and cleanup are recorded in `target/headstart/dialog-gui-acceptance.json`; this is partial interaction evidence, not full GUI acceptance.

## 2026-10-19 M119 source and native update

`DialogView` now accepts literal and localized accessible names, with visible
title and trigger fallbacks. Four focused Dialog unit tests and the Playground
package check passed. The macOS story rendered the modal; its accessibility
tree exposed a named `dialog` node with the title, body, and action controls.
Focus entry/containment/restoration and screen-reader speech remain unverified.
Evidence: `target/headstart/evidence/dialog-name-m119-20261019.json`.

## 2026-10-19 M127 bounded native focus probe

On macOS, Tab moved visible focus through Cancel -> Confirm -> Close -> Cancel without an outside Playground control receiving focus. Starting from Cancel, Shift+Tab wrapped to Close and then moved through Confirm and Cancel; screenshots showed each focus ring. Escape dismissed the dialog and displayed `Cancelled: no action was taken.` Pressing Return immediately, without an intervening Tab or pointer action, reopened the dialog, showing keyboard activation returned to the Open dialog trigger. The accessibility API reported the Playground window rather than a focused child control, so direct child focus was not established. Screen-reader speech, theme, locale and cross-platform behavior remain unverified. Evidence: `target/headstart/evidence/dialog-focus-m127-20261019.json`.
