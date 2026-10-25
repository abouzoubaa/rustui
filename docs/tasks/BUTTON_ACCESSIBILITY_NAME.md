# Button accessible-name localization

`ButtonView::localized_accessibility_label(...)` resolves its catalog message
through the active `RenderContext`. It rejects blank keys and falls back to the
visible Button label when the catalog message is blank or unavailable. An
explicit literal accessibility label becomes the fallback and remains exact,
including valid surrounding whitespace. Re-localizing the Button keeps its
original fallback.

Focused unit coverage checks blank-key rejection, visible and explicit fallback
selection, catalog result/fallback resolution, and literal-label preservation.
Validation:

- `./scripts/cargo-headstart test --quiet -p rustui --lib components::basic::button::tests:: --locked --offline` — 9 passed, 0 failed.
- `rustfmt --check --edition 2024 crates/rustui/src/components/basic/button.rs` — passed.
- `git diff --check` — passed.

No GUI or native accessibility acceptance was run.

## M585 native follow-up

The canonical Playground Button story now resolves its increment control's
accessible name from the active catalog. On macOS, the native accessibility
tree exposed `Increase counter`, `Augmenter le compteur`, and `زيادة العداد`
after English, French, and Arabic selection without reselecting the story.
Pointer activation incremented the count, and clicking the disabled control left
the count unchanged. Keyboard activation, VoiceOver speech, broader visual, and
cross-platform acceptance remain open. Evidence:
`target/headstart/evidence/m585-button-native-ax-interaction-20261021.json`.

### M585 keyboard follow-up

On the same macOS Playground story, CUA sent Tab, Return, and Space while the
window remained active. Count stayed at 1 and the event log did not change; the
native accessibility tree continued to report the window as the focused UI
element, so the attempt did not establish that keyboard focus reached the
Button. Keyboard activation remains unverified. Build and attempt details:
`target/headstart/evidence/m585-button-keyboard-followup-20261021.json`.
