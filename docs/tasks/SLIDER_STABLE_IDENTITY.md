# Slider stable identity

Two sibling Sliders with the same range and step can share one retained GPUI
`SliderState` if their fallback IDs use only configuration values. Implicit
Slider IDs use `RenderContext`'s owner-scoped sibling-slot allocator so siblings
keep separate state and each slot remains stable across renders.

Caller-provided `.id(...)` values remain unchanged and bypass fallback
allocation. This is source/test evidence; native interaction, focus,
accessibility, and cross-platform acceptance remain separate.

Focused evidence: `target/headstart/evidence/m516-slider-sibling-identity-tests-20261021.log`.
