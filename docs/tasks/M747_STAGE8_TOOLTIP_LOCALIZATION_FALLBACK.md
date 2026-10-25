# M747 — Stage 8 / preserve Tooltip fallback for unresolved localization

`Tooltip(text, content).localized(key)` now retains the original literal text
as its fallback, including across repeated localization overrides. When the
catalog returns a blank value or echoes the key (including surrounding
whitespace), the wrapper's accessible description and native popup use that
literal text. A resolved translation remains preferred.

Regression: `untranslated_localized_tooltip_keeps_its_original_literal_fallback`.
Validation: `./scripts/cargo-headstart test --locked --offline -p rustui --lib
untranslated_localized_tooltip_keeps_its_original_literal_fallback` passed
(1 passed, 739 filtered out). Scoped rustfmt and diff whitespace checks passed.

This is source-level evidence only. Native hover, touch, assistive-technology,
locale-switching, RTL, and cross-platform acceptance remain open; see the
Tooltip contract in `O2_TOOLTIP.md` and Stage 8 overlay acceptance gate in
`ROADMAP_COMPLETION_GATES.md`.
