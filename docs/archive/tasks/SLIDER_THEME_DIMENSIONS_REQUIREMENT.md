# Slider theme thumb dimensions

The Slider renderer reads the resolved `slider/sliders` style's `width` and
`height` as required thumb dimensions. Those properties were included in the
renderer capability record, but a malformed theme could omit one and pass
typed component validation before the renderer's `expect` failed. Theme
validation now requires both dimensions on every `sliders` rule.

The focused regression removes `width` from the valid Slider rule and verifies
that theme validation rejects the document. Native visual acceptance remains
separate.
