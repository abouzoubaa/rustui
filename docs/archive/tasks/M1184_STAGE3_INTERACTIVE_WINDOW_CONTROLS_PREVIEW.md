# M1184 — Stage 3 interactive window-controls preview

## Scope

Replace the static Playground `window_controls` labels with a public RustUI
Button composition for the illustrative Close, Minimize, and Zoom traffic
lights. The owner stores one active/inactive preview state. Clicking a light
marks the preview active; a separate control simulates an inactive window. The
visible labels and accessible names reflect that state.

The preview explicitly says it is illustrative: button clicks do not perform
system window actions, and no native titlebar or window-control renderer is
implemented. This does not claim native chrome fidelity.

## Verification

`./scripts/cargo-headstart test -p rustui-playground --lib stories::window_controls_gallery::tests::window_control_buttons_reflect_owner_active_state --locked --offline`
passed 1/1. Cargo reported the existing `block 0.1.6` future-incompatibility
notice. The regression was extended to deactivate the preview before each
Close, Minimize, and Zoom action and assert reactivation; each owner action now
sets `active = true` before recording the selected control.

Native visual, keyboard, accessibility, localization, and platform acceptance
remain open.
