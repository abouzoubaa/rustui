# M900 — Stage 7 / prepare Slider accessibility-name patch proposal

The M595 audit found that the four HSLA sliders inside ColorPicker's popup
have no accessible names because the pinned GPUI Components Slider exposes no
accessible-label setter. A review-only proposal adds
`Slider::accessibility_label(...)` and forwards it through the existing
BaseSlider `aria_label` path.

Proposal: `patches/proposed/gpui-component-slider-accessibility-label.patch`
Checker: `scripts/check_gpui_component_slider_accessibility_label_patch.py`

The isolated checker passed pinned package/source hash checks, exact zero-fuzz
disposable patch application, postimage invariants, and rustfmt. The patch has
not been applied. Explicit review and approval are required before applying it
or changing RustUI to call the new API. GPUI compilation, native GUI behavior,
VoiceOver, and cross-platform accessibility remain open.
