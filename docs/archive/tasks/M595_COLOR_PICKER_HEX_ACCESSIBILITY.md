# M595 — ColorPicker hex-field accessibility

## Contract

The RustUI-owned hex color input in the ColorPicker HSLA popup must expose a
non-empty accessible name. Resolve `color_picker.hex_value` from the active
RustUI catalog and use the English fallback `Hex color value` when the message
is missing or blank. The Playground ships English, French, and Arabic values.

## Scope and limits

This labels only the RustUI-owned hex input. The four HSLA sliders still have
single-letter visual labels; the pinned GPUI Components Slider API does not
expose an accessible-label setter. Native popup accessibility, VoiceOver,
locale-switching, RTL, and cross-platform acceptance remain open.

## Verification

The focused ColorPicker regression is included in the RustUI library suite
(681/681 passed). The locked/offline workspace all-targets check and Playground
build passed; locale validation passed. Native macOS accessibility-tree checks
observed the English, French, and Arabic names in the HSLA popup. The four
upstream sliders remained unnamed. Evidence:
`target/headstart/evidence/m595-colorpicker-accessibility-native-20261021.json`,
`target/headstart/evidence/m596-m595-rustui-library-suite-20261021.log`,
`target/headstart/evidence/m596-m597-workspace-all-targets-check-20261021.log`,
and `target/headstart/evidence/m595-m597-playground-build-20261021.log`.
