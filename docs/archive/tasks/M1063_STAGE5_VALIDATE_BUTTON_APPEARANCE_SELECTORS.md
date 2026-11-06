# M1063 — Stage 5 / reject unsupported Button appearance selectors

## Problem

Button style rules could accept an arbitrary `when.appearance` value even
though runtime resolution only matches `any`, `light`, and `dark`. For example,
an otherwise valid `system` rule could silently remain unresolvable.

## Change

`ButtonStyleRule::validate` now rejects appearance selectors outside
`any`, `light`, and `dark`, aligning serialized Button rules with the runtime
resolver and the existing typed component-rule validation.

## Focused evidence

`./scripts/cargo-headstart test -p rustui --lib button_theme_rejects_unresolvable_appearance_selectors`

Result: 1 passed. The regression rejects a Button rule using `system`.
Scoped `git diff --check -- crates/rustui/src/theme/schema.rs` passed. Native
visual, accessibility, and cross-platform theme acceptance remain open.
