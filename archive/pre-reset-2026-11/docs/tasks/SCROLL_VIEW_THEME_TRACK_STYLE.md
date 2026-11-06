# M363 — Stage 5 / ScrollView track theme mapping

RustUI now applies the declared `scroll_view` track `border` color through the
pinned `gpui_component::scroll::Scrollbar` API's `ScrollbarTrackStyle::border_color`.
It also applies the resolved `corner_radius` to the upstream thumb's
`ScrollbarThumbStyle::radius`, using the SVG measurement only when the style
does not declare that property. Both horizontal and vertical renderer
capabilities now claim these two mapped properties.

The `width` and `height` values in these records are retained as source-artwork
extents. They do not define the thumb's visible thickness or its scroll-axis
length: the profile records those thumb measurements separately, and the
pinned scrollbar derives thumb length from viewport and scroll geometry. The
unapplied dimensions therefore remain outside the RustUI capability contract.
The independent dark `radio_button.blend_mode` limitation is unchanged.

Evidence:

- `vendor/gpui-base-0.7.3/src/scrollbar.rs` exposes `ScrollbarTrackStyle::border_color`
  and `ScrollbarThumbStyle::radius`.
- `crates/rustui/src/layout/scroll_view.rs` reads and applies the resolved
  properties for both axes.
- `scripts/verify_theme_adapter_coverage.py` statically guards those reads,
  applications, and the known source-artwork dimension limit.
- `target/headstart/evidence/theme-adapter-coverage-scrollview-m363-20261021.log`
  records the verifier result.
- `target/headstart/evidence/scrollview-track-theme-m363-check-20261021.log`
  records a successful locked `rustui --lib` check.
- `target/headstart/evidence/scrollview-track-theme-m363-test-20261021.log`
  records the focused resolver regression, passed 1/1.

The resolver regression confirms that `border` and `corner_radius` are not
reported as unapplied for either track. This source-level mapping does not
establish native visual acceptance.
