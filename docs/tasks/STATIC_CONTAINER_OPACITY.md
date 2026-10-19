# Static Container opacity

`Container(content).opacity(value)` sets a fixed opacity on the GPUI box that
contains both the styled surface and its RustUI child. Values must be finite and
within `0..=1`; invalid values panic at the builder call, consistent with
`Container`'s existing numeric builders. Without the builder, the box has no
opacity override and remains fully opaque. A later `.opacity(value)` call wins.

This is static styling. It does not schedule an animation, change child event
handling, or provide a second public opacity wrapper. `AnimatedOpacity` remains
the separate one-shot mount animation API.

## Container shadow

`Container(content).shadow(offset_x, offset_y, blur_radius, spread_radius, color)`
sets one outer GPUI box shadow. Offsets, blur, and spread use logical pixels;
offsets must be finite, blur must be finite and nonnegative, and spread may be
negative to match GPUI's shadow geometry. The caller supplies an `Hsla` color.
No shadow is set by default, and a later `.shadow(...)` call replaces the
previous one. The Playground animation story places a no-shadow container beside
a configured shadow for visual comparison; this source change does not establish
native visual acceptance.

Focused unit coverage checks the default, valid endpoints and midpoint, and
rejection of out-of-range or non-finite values. The existing Playground animation
gallery includes fully opaque and half-opacity examples. The log at
`target/headstart/milestone43-45-tests.log` records 120 passing tests
across 12 binaries, with 0 failures. The native M45 screenshot showed the
`1.0` sample brighter and the `0.5` sample dimmer across their labels and
surfaces; the reviewed binary hash is recorded in
`target/headstart/milestone45-static-opacity-gui.json`. Pixel alpha was not
measured, and the screenshot does not verify child pointer or event behavior.
