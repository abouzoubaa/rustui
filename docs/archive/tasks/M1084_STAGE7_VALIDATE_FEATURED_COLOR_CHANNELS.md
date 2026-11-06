# M1084 — Validate featured ColorPicker channels

Before accepting custom featured swatches, require each HSLA channel to be
finite and within `0..=1`. Reject invalid values at the RustUI API boundary.

The focused regression covers one valid swatch, NaN, infinity, and out-of-range
channels; it passed 1/1. Native popup interaction and selection acceptance
remain open.
