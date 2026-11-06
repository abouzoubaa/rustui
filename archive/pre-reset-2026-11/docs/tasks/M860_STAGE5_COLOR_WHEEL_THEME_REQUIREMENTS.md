# M860 — Stage 5 / require ColorPicker wheel fill and radius

Theme validation for the `color_picker/color_wheel` style now requires both
`corner_radius` and `background`, matching the renderer capability contract.
The focused regression removes each property in turn and passed 1/1; scoped
diff validation passed. Whole-file `rustfmt --check` still reports existing
formatting differences elsewhere in the already-dirty `schema.rs`; the new
lines were not among them. See `crates/rustui/src/theme/schema.rs`.

Native visual and accessibility acceptance remain open.
