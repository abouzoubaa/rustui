# M1069 — Stage 7 / reject slider steps with non-finite snapped endpoints

`Slider::step` now checks both configured range endpoints against GPUI's
quotient-and-rounded-product snapping calculation. It rejects positive finite
steps that still overflow during snapping, including subnormal steps and large
range values. Focused regressions for the subnormal rounding overflow and
large-value snapped endpoint overflow passed (2/2). Broader native interaction
and accessibility acceptance remain open.
