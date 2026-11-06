# TextField and Slider disabled theme audit

## TextField

The public `TextFieldView::disabled` path passes disabled state to GPUI
Components `Input`. Its visible value text inherits the element text color, then
the pinned input renderer applies its normal disabled dimming. RustUI now supplies
the active profile's `disabled_text` token for that text only when disabled.
Enabled text keeps the component's resolved foreground. Placeholder text keeps
its separate muted-foreground treatment, and the input's built-in disabled
background and prefix/suffix dimming remain in place.

The focused light/dark token test passed. Evidence:

- `target/headstart/evidence/textfield-slider-disabled-theme-focused.log`

## Slider

`SliderView::accessibility_label` supplies an assistive-technology name and does
not render a visible label. Its disabled track and thumb use the declared
`disabled: muted_foreground` theme role, which the macOS 27 profile resolves to
`secondary_text`; existing track and thumb opacity preserve the disabled
dimming. There is no visible label color to map to `disabled_text`, so this
source path remains unchanged. The existing role/color, disabled-action, and
accessibility tests passed in both appearances where applicable.

- `target/headstart/evidence/textfield-slider-disabled-theme-slider-regression.log`

No GUI acceptance was performed.
