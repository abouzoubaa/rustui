# M1123 — normalize ColorPicker hue endpoints for swatch selection

Hue values 0 and 1 map to the same RGB color, but exact Hsla equality treated
them as different when checking whether a featured swatch was selected. The
comparison now normalizes hue modulo one before matching.

Focused regression:

```text
./scripts/cargo-headstart test -p rustui hue_endpoints_match_the_same_selected_color
```

The command exited successfully. Scoped `git diff --check` passed. Native
interaction and accessibility acceptance remain open.
