# Slider numeric contract audit

Scope: `crates/rustui/src/components/input/slider.rs` and its in-file unit tests.

- Bounds: `SliderView::new` rejects non-finite or non-ascending endpoints;
  controlled values are clamped before entering retained `SliderState`.
- Step: `.step` rejects non-positive and non-finite values. Accessibility
  SetValue rounds to a multiple of step and clamps to bounds. Pointer input
  uses GPUI Base's same zero-anchored rounding rule.
- Finite values: NaN previously survived `f32::clamp` and entered retained
  state. It now resolves to the finite minimum endpoint. Infinities clamp to
  the corresponding endpoint.
- Controlled updates: the current binding value is re-read each render and
  synchronized into retained state; Change events write back through the
  binding writer. Change-event values are now clamped to the current finite
  range before the binding writer runs, so an upstream f32 step-rounding
  overflow cannot publish an infinity or out-of-range value to the caller.
  This is needed because GPUI Base pointer handling computes
  `(value / step).round() * step` in `f32`, which can overflow for valid
  finite ranges and positive steps.
- Focused coverage: `controlled_slider_value_is_finite_and_within_bounds`
  covers NaN, infinities, out-of-range finite values, and an in-range value.
  `slider_event_values_are_finite_and_within_current_bounds` and the retained
  writer regression cover event sanitization and the current owner binding.
  Existing tests cover SetValue step rounding/clamping, disabled SetValue
  rejection, and finite positive step validation by constructor behavior.

Validation: `./scripts/cargo-headstart test -p rustui --lib
components::input::slider::tests` passed (14 passed, 0 failed); source formatting
check passed with `rustfmt --check --edition 2024
crates/rustui/src/components/input/slider.rs`. Full native GUI, accessibility,
and cross-platform gates remain open.

## M435 — bounded retained Change values

Retained Change events are clamped to the current finite range and map NaN to
the lower endpoint before updating the current owner Binding. This blocks
upstream step-rounding overflow and endpoint overshoot from entering controlled
state. The focused Slider suite passed 14/14; rustfmt passed. Native pointer
interaction, accessibility, and cross-platform acceptance remain open.
Evidence: `target/headstart/evidence/slider-numeric-contract-audit-20261021.log`.
The integrated locked/offline workspace all-targets check exited 0; evidence:
`target/headstart/evidence/workspace-all-targets-check-m427-m435-20261021.log`.

## M505 — handle positive subnormal Slider steps

Step normalization now uses `f64` intermediates so the reciprocal of a positive subnormal `f32` step cannot overflow while deriving the normalized step. The focused regression `slider_accessibility_set_value_handles_subnormal_steps` passed, and the complete Slider unit suite passed 15/15. Evidence: `target/headstart/evidence/m505-slider-regression-tests-20261021.log`. Native pointer interaction, accessibility, and cross-platform acceptance remain open.

## M522 finite span

The accepted range must also have a finite `f32` span. The pinned GPUI linear slider computes `max - min` in `f32`; finite endpoints alone can overflow that span and produce invalid percentages. The regression rejects `-f32::MAX..=f32::MAX` and accepts `-f32::MAX..=0.0`. Focused Slider tests passed 19/19. Evidence: `target/headstart/evidence/m522-slider-finite-span-tests-20261021.log`.
