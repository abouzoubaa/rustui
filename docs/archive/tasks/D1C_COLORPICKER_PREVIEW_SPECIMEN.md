# D1c ColorPicker preview specimen

The canonical compiled-preview gallery now has an opt-in ColorPicker specimen
using the public RustUI `ColorPicker` API and its controlled
`Binding<Option<Hsla>>`. The descriptor stores an optional color as a `has_color`
boolean plus normalized HSL and alpha number properties, each constrained to
`[0, 1]`.

Color feedback writes all five values in one host edit. Writes require the
current preview stamp, and the preview schema validates the complete update.
The generic inspector edits the label; the ColorPicker control owns the color
channels. The specimen registers no preview events or actions. Descriptor
export/import preserves both selected and unset colors.

The focused regression passed 1/1, including bounded schema values, atomic
feedback, stale-stamp and out-of-range rejection, and selected/unset
round-trips. Rustfmt passed. An isolated locked/offline Playground all-targets
check exited 0; see
`target/headstart/evidence/d1c-colorpicker-all-targets-20261020.log`.
No GUI or native accessibility acceptance was run.
