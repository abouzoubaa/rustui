# M853 — Stage 7 Slider identity across reconfiguration

Implicit Slider identity included the configured range and step. Changing
either value therefore created a new retained Slider entity instead of
retaining the control and applying its existing range/step refresh path.

Implicit identity now uses the stable owner slot and Slider base. Explicit IDs
remain unchanged. The focused regression confirms sibling slots stay distinct
and that a Slider keeps its identity when its range and step change.

Focused evidence:

```text
./scripts/cargo-headstart test -p rustui --lib implicit_slider_identity_distinguishes_siblings_and_is_stable_by_slot --locked --offline
1 passed; 773 filtered out
```

`git diff --check` passed. The scoped rustfmt check still reports an unrelated
pre-existing formatting difference in Slider's localized label resolution;
that shared edit was left unchanged.

This is source-level identity evidence. Native pointer/drag, keyboard,
accessibility, and cross-platform acceptance remain open.
