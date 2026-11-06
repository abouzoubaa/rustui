# M1150 — Stage 7: name an unlabeled Slider

`Slider` always renders the `Role::Slider` accessibility node. Its optional
accessible-name override now falls back to the nonblank generic name `Slider`
when omitted; caller-provided names remain unchanged. This prevents the public
component from exposing an unnamed slider while preserving the optional
override builder.

The focused source regression passed:

`./scripts/cargo-headstart test --locked --offline -p rustui --lib components::input::slider::tests::slider_without_an_override_uses_a_nonblank_accessible_name -- --exact`

This verifies source-level name resolution only. Native accessibility-tree,
screen-reader, and cross-platform acceptance remain open.
