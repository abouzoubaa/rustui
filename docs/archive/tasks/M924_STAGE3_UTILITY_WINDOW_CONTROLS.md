# M924 — Stage 3 / add representative Utility Window tab controls

The Utility Window preview retains M922's five category tabs and adds
representative public RustUI controls to their bodies:

- Color Wheel, Swatches, and Crayons use `ColorPicker`.
- Sliders shows accessible Red, Green, and Blue channel sliders.
- Images shows the existing embedded Playground sample image with an
  accessible name.

The focused tab-to-panel mapping regression passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui-playground \
  utility_window_gallery_matches_the_reference_tab_groups --lib
```

Rust 1.97.1 `rustfmt --check` passed for the changed source. This verifies
source composition and the tab mapping; native visual, interaction, and
accessibility acceptance remain open.

Added `utility_window_gallery_names_accessible_controls` to protect the
accessible Red, Green, and Blue slider names and the sample image name. The
focused check did not reach test execution: rustc reported `E0463` for missing
`gpui`, `gpui_component`, `rustui_core`, `rustui_i18n`, `chrono`, `gpui_base`,
and `serde` crates. The new regression remains unverified until the filtered
check can build successfully.
