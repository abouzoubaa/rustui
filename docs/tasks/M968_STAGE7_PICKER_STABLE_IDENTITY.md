# M968 — Picker identity across localization changes

Stage 7 / input state retention.

Picker implicit identity now derives from its owner-scoped sibling slot rather
than its visible label. A label change can preserve retained Picker state,
while two same-label sibling Pickers remain distinct.

The focused identity tests passed 3/3. Rust formatting and scoped whitespace
checks passed. Runtime rerender across locale changes, native accessibility,
keyboard/focus, VoiceOver, RTL, visual, and cross-platform acceptance remain
open.
