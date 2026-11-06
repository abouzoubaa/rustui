# M1118 — Stage 7 / match Slider accessibility and pointer step rounding

Slider accessibility `SetValue` normalized its value using `f64` division and
rounding, while the pinned GPUI pointer interaction divides, rounds, and
multiplies in `f32`. Near a half-step boundary these paths could choose adjacent
values.

Accessibility normalization now uses the same `f32` operations when the
quotient is finite. A focused regression reproduces the boundary where the
previous `f64` result differed and asserts that accessibility chooses the same
value as pointer interaction. A `f64` fallback remains for direct helper calls
with subnormal steps; Slider construction rejects steps that would overflow
the configured `f32` range.

`./scripts/cargo-headstart test -p rustui
slider_accessibility_set_value_matches_pointer_rounding_in_range` passed
(1 passed, 841 filtered out). `git diff --check --
crates/rustui/src/components/input/slider.rs` passed.

Native pointer, keyboard, visual, screen-reader, and platform acceptance
remain open.
