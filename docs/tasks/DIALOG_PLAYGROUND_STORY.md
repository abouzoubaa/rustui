# Dialog Playground story

## Scope

Add a dedicated RustUI Playground demonstration for the public trigger-driven `Dialog` API. The story opens one dialog and displays the result of its Confirm or Cancel action.

## Acceptance

- The existing `save_dialog` catalog entry opens the dedicated story.
- The story uses only public `rustui::prelude` APIs.
- Confirm and Cancel each produce visible result text.
- The demonstration describes the single-topmost dialog contract. It makes no claim about nesting, controlled visibility, owner-removal closure, or patch-backed behavior.
- Build and formatting checks pass. A bounded macOS interaction probe reached the `save_dialog` story and observed the accessibility-tree result text change to `Confirmed: action accepted.` after Return and `Cancelled: no action taken.` after Escape. The screenshot remained an unchanged Button story while the accessibility tree reflected Dialog state, so visible rendering, modal focus, accessibility semantics, theme appearance, and locale behavior remain unverified. Exact steps and cleanup are recorded in `target/headstart/dialog-gui-acceptance.json`; this is partial interaction evidence, not full GUI acceptance.
