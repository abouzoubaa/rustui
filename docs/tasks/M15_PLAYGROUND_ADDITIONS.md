# M15 Playground foundation additions

`apps/rustui-playground/src/stories/foundation_additions.rs` exposes
`foundation_additions() -> impl View` using RustUI's public API. It is retained
as a keyed child so its `ThemeFile` and visible reload result survive parent
redraws. The light/dark controls use `reload_if_changed`; the status text
reports open, reload, and unchanged outcomes and is included in each control's
accessibility label. The 200px wide sample also shows a main-axis Spacer with an
explicit horizontal minimum, horizontal and vertical Dividers, and the physical
left/right values from resolving the same `LogicalInsets` under LTR and RTL.

## Root integration

In `apps/rustui-playground/src/stories/mod.rs`, declare
`mod foundation_additions;` and import
`foundation_additions::foundation_additions`. Add
`(selected == StoryId::Layout && self.w1_step() == 0).then(foundation_additions)`
to the normal `VStack!` content before `StoryCanvas::new(...)`. This keeps the
examples in the ordinary Layout story and out of the compact W1 sequence.

The theme buttons read `themes/macos27.json` from the workspace and apply the
chosen appearance through `ThemeFile::reload_if_changed`. This is a source-level
demonstration; no GUI verification is claimed here.
