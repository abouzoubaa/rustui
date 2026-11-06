# M1185 — Stage 5 / require positive scrollbar dimensions

Date: 2026-10-23

Custom theme SVG metadata could set scrollbar thumb thickness or the 8x minimum thumb length to zero. The renderer passed those values to GPUI as track width and minimum thumb length, allowing an invisible scrollbar. Zero remains valid for thumb radius and fill opacity.

The shared metadata reader now requires strictly positive finite values for thickness and minimum length, falling back to the existing defaults when invalid. Radius and opacity retain the nonnegative rule, including zero. The existing theme `corner_radius` override remains intact.

Focused regression: `./scripts/cargo-headstart test -p rustui --lib layout::scroll_view::tests::scrollbar_dimensions_fall_back_from_zero_but_opacity_and_radius_allow_zero --locked --offline` — passed (exit status 0).

This establishes metadata fallback behavior only. Native scrollbar visual, keyboard, accessibility, and cross-platform acceptance remain open.
