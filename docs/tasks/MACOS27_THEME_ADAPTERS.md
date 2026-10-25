# macOS 27 theme adapter correction

## Source trace

Playground activates the embedded `themes/macos27.json` through
`activate_theme`. That function applies `platform_snapshot` to GPUI Components'
shared theme and stores the full document as `ActiveTheme`. A component JSON
style affects rendering only when its RustUI renderer resolves and applies it.

The pack has 41 component records and 419 style rules. Of the 27 records mapped
to RustUI renderers, 24 contain styles and three (`toolbar`, `sidebar`,
`tab_bar`) contain no body styles. `menu_bar` also has no styles and has no
RustUI renderer. Its absence is classified as reference only.

## Corrected framework paths

- `ThemeProfile::colors(None)` resolves the profile's declared default
  appearance, while explicit `Some(Light)` and `Some(Dark)` select their
  requested palettes. The isolated public API regression clones the parsed
  theme's `platform_snapshot`, sets its default to Dark, and checks all three
  cases.
- `styled_element` applies resolved typography to GPUI's font family, size,
  and weight refinement. The light text-body regression checks those values
  against the resolved macOS 27 style.
- `TextFieldView` uses the same Small or Large choice for the GPUI input and
  the theme selector. The previous `.small()` input queried the Large rule.
  Both macOS 27 body widths are covered by a resolver regression test.
- Checkbox and radio unchecked borders now use the pack's declared `border`
  role. Their checked or selected style and rectangular artwork dimensions
  still come from component rules. Unchecked fill and focus use the mapped
  platform input surface and ring; the pack has no component rule for them.
- Toolbar, sidebar, and tab bar consume their declared profile geometry:
  toolbar height and compact spacing, sidebar width, and control height.
  These records have no body style rules, so their appearance colors come
  from the GPUI role adapter.
- `verify_theme_adapter_coverage.py` now checks each styled mapped record's
  renderer query and resolved-style consumer, plus the three profile metric
  paths. It continues to verify all pinned GPUI `ThemeColor` role assignments.

## Renderer audit and remaining limits

The 24 styled mapped records have RustUI query paths in button, basic,
input, navigation, data, overlay, and scroll view renderers. Source-linked
validation detects a missing query or consumer; it does not prove visual
fidelity, every conditional branch, or that every artwork property is drawn.
The existing `ThemeCoverageReport` retains rule and property gaps rather than
blocking activation.

Selector reachability remains limited for source-only variants: Text and Label
currently select opaque material; Icon selects its default `icon` part and
rest interaction; Image selects generic `image`, while utility `images` is a
different source part. TextField's `control_search_field` parts are not used
by its generic input wrapper. The pack also has reference-only compositions
listed in `theme/registry.rs`; they are not RustUI renderers. These records
need an explicit component API and renderer semantics before claiming runtime
coverage. The Slider utility selector is implemented in its renderer; its
separate accessibility action change remains owned by the Slider workstream.

No GUI acceptance or Cargo validation was run in this workstream. The Python
adapter guard was run directly and reported 139 assigned GPUI color roles,
24 styled renderer paths, and three profile renderer paths.

## Follow-up selector reachability review

The current builders expose opt-in selectors for vibrant text and labels,
clear-icon artwork, and utility images. Their renderers resolve the matching
macOS 27 records; remaining limits below concern properties or state they do
not apply.

- **Text and Label vibrant material:** `TextView` and `LabelView` expose an
  explicit `.vibrant()` opt-in and select appearance/material-specific parts;
  their renderers apply the resolved foreground and typography. Ordinary text
  stays on opaque rules by default. `TextView` accepts the pack's `blend_mode`
  as metadata but does not apply blending. These APIs select text styling;
  they do not draw or inherit the surface behind the text, and native
  composition is not claimed.
- **Icon clear/pressed:** `IconView::clear()` and `IconView::clear_pressed()`
  select the pack's `clear` rule for its rest and pressed states. The renderer
  uses the rule's vector path and foreground styling. `IconView` remains
  noninteractive; the enclosing control supplies the pressed state and owns
  its action, focus, and accessibility semantics.
- **Image utility rules:** `ImageView::utility()` selects the `images` rule
  with `chrome = utility`; the renderer applies its resolved dimensions and
  container styling. Ordinary images still use the generic `image` rule.
  Utility classification is an explicit builder choice rather than an
  inherited chrome context.
- **TextField `control_search_field`:** `SearchField` creates the existing
  `TextFieldKind::Search`, but the renderer delegates interaction to GPUI
  Components' generic `Input` and resolves `search_field/list_search`.
  The selector inputs are available: RustUI owns disabled and bound-text
  state, and the pinned public `InputState::focus_handle(cx)` plus GPUI
  `FocusHandle::is_focused(window)` expose focused/rest state. The six
  `text_field/control_search_field` rules cover only light appearance and
  distinguish disabled, focused, and rest states with text present/absent.
  Reachability alone is insufficient to apply the record: it declares a
  filter, blend mode, vector path, and focused cursor geometry, while the
  current renderer applies ordinary layout/color/typography and its generic
  `IconName::Search`; it does not interpret those artwork properties. The
  pinned GPUI `Styled`/text APIs have no blend-mode property, and the theme
  resolver intentionally refuses unconsumed artwork. GPUI's existing SVG
  icon path can draw a vector path, but it does not implement this record's
  filter/blend composition or its full control geometry. Do not select this part
  as complete coverage until a framework-owned renderer can consume its
  artwork; verify state selection with a focused resolver test, then verify
  the complete appearance in a native visual probe after the renderer is
  available. No vendor edit or Playground-only override closes this gap.

- **Text `blend_mode`:** `.vibrant()` selects the correct appearance/material
  rule and the renderer applies its foreground and typography, but GPUI's
  pinned public text/style APIs provide no blend-mode operation. The resolved
  `plus-darker` and `plus-lighter` modes require composition against the
  surface behind the text; approximating them with the resolved foreground
  loses that background-dependent behavior. Applying them requires an
  approved framework composition capability (or an upstream public API that
  provides equivalent semantics). After that boundary is available, verify
  both light/dark modes over representative surfaces in a native visual
  probe; source resolution alone is not acceptance.

These are framework integration gaps rather than Playground configuration
gaps. Playground can verify the framework after these contracts are designed,
but a Playground-only style override would conceal rather than resolve them.


## M415 — Exclude test-only renderer markers

`check_renderer_calls` now strips Rust `#[cfg(test)]` modules before checking source markers. A regression with a test-only label renderer confirms that query and resolved-style consumer markers in tests cannot satisfy production coverage. The focused verifier regression passed 1/1; the complete static verifier exited 0. This remains source coverage evidence, not visual acceptance. Evidence: `target/headstart/evidence/theme-adapter-coverage-m415-unit-20261021.log` and `target/headstart/evidence/theme-adapter-coverage-m415-verify-20261021.log`.

## M449 — selector reachability note correction

Corrected stale selector-reachability notes in the Stage 5 adapter audit. Source review and scoped whitespace checks passed; no Cargo build or GUI acceptance was run.

## Stage 5 follow-up — adapter API boundary

The `control_search_field` selector state is publicly observable, correcting
the earlier claim that GPUI `InputState` focus was unavailable. No source
change was made: the existing GPUI SVG path API can draw vector artwork, but
the adapter has no filter/blend composition or complete control renderer.
Text `blend_mode` has the same pinned-GPUI composition boundary. Next evidence
is a focused state-to-rule resolver regression alongside an approved
artwork-capable framework renderer, followed by native light/dark visual
probes; neither source inspection nor a static verifier can establish visual
parity.

## Stage 5 follow-up proposal — M468 or later / SearchField state renderer

The six light `text_field/control_search_field` rules have identical ordinary
width, height, corner radius, background, and foreground values. The
state-dependent values are artwork and caret properties: text presence changes
the filter and vector path, while focused rules add cursor color, width,
height, and radius. The pinned public GPUI `Styled` and text APIs expose no
filter/blend composition operation, and `InputState` exposes no caret color or
geometry customization. GPUI can draw SVG paths, but the current RustUI input
renderer has no contract that consumes the pack's filter graph and cursor
properties. Resolving different selectors would therefore not apply the
state-specific appearance; no source change is justified at this boundary.

Proposed next task: **M468 or later — Stage 5 / approved SearchField state
renderer**. Define a framework-owned adapter for the filter/blend and caret
properties, route disabled/focused/text-presence selectors through the retained
input state, then add focused coverage for all six rules and native light/dark
visual probes. Keep SVG path drawing and ordinary shared colors/dimensions
distinct from full artwork and caret support; do not claim parity before the
renderer and probes exist.

## M501 — GPUI secondary role follows the toolbar token

The top-level `secondary` theme role resolves through `toolbar` to
`toolbar_background`, but `apply_profile` assigned the elevated panel surface.
The adapter now maps GPUI `secondary` to the toolbar color. Its focused
light/dark regression passed 1/1 with:
`./scripts/cargo-headstart test -p rustui --lib adapters::themes::tests::secondary_background_matches_macos27_toolbar_role`.
Evidence: `target/headstart/evidence/m501-theme-secondary-background-20261021.log`.
This is source/test evidence only; native light/dark visual acceptance remains
open.

## M556 metric representation validation

Body font size and medium/large radii must be positive, finite, and representable as f32 before GPUI theme state changes. Focused locked/offline tests passed 3/3 in `target/headstart/evidence/m556-theme-metric-representability-20261021.log`. Native visual acceptance remains open.


## M560 material opacity representability
`apply_profile` validates public `ThemeProfile.materials.material_opacity` before changing GPUI theme state: the value must be finite, within `[0, 1]`, and must not underflow to zero when narrowed to `f32`. Boundary values 0 and 1 and an ordinary fractional value remain accepted. Focused adapter test passed 1/1 and the theme adapter suite passed 4/4 in `target/headstart/evidence/m560-theme-material-opacity-validation-20261021.log` and `target/headstart/evidence/m560-theme-adapter-suite-20261021.log`. Native visual acceptance remains unverified.

## M567 radius f32 representability
`Radii::validate` now rejects every radius that is nonpositive, nonfinite, overflows to f32 infinity, or underflows to f32 zero. Focused overflow, underflow, and representable-boundary regressions passed 3/3 in `target/headstart/evidence/m567-theme-radius-representability-20261021.log`. Native visual acceptance remains unverified.

The published `themes/theme.schema.json` radii fields also use `positiveMeasure`, matching the runtime requirement that every profile radius be greater than zero. Zero-radius rejection has a focused schema regression.


## M588 spacing f32 representability
`Spacing::validate` now rejects values that overflow to f32 infinity or positive values that underflow to f32 zero, matching the range consumed by the resolver. Zero remains valid. Focused regressions exercise overflow, underflow, zero, and representable f32 boundaries across all six spacing tokens. Evidence: `target/headstart/evidence/m588-theme-spacing-f32-representability-20261021.log`. Native visual acceptance remains unverified.

## M589 dimension f32 representability
`Dimensions::validate` now rejects positive finite f64 values that overflow to f32 infinity or underflow to f32 zero before theme resolution narrows them. Focused schema regressions cover overflow, underflow, and representable f32 boundaries; 7/7 schema tests passed. The locked/offline workspace all-targets check passed with existing example warnings and a future-incompatibility notice for `block 0.1.6`. Formatting and diff checks passed. Native visual acceptance remains open. Evidence: `target/headstart/evidence/m589-theme-dimensions-f32-representability.log` and `target/headstart/evidence/roadmap-workspace-check-after-m589-20261021.log`.
## M590 typography f32 representability

`Typography::validate` now rejects base font sizes and named style sizes that overflow to f32 infinity or underflow to f32 zero before GPUI consumes them. Zero and negative values remain rejected by the existing positive-number checks. Focused schema tests cover both size groups and representable f32 boundaries; 2 new regressions passed. The locked/offline workspace all-targets check passed. Native theme visual acceptance remains open. Evidence: `target/headstart/evidence/m590-theme-typography-f32-tests-20261021.log` and `target/headstart/evidence/roadmap-workspace-check-after-m590-20261021.log`.
## M591 border and focus width f32 representability

`Borders::validate` now validates `thin` and `medium` widths against the f32 values used by Checkbox, Radio, and Toggle. `Accessibility::validate` applies the same check to `focus_width`, which Checkbox and Radio consume. Overflow to infinity and positive underflow to zero are rejected; explicit zero remains supported. Focused schema regressions cover both groups and representable boundaries; the locked/offline workspace all-targets check passed. Native theme visual acceptance remains open. Evidence: `target/headstart/evidence/m591-theme-border-focus-width-f32-tests-20261021.log` and `target/headstart/evidence/roadmap-workspace-check-after-m591-20261021.log`.

## M716 nonempty theme-role map

The published `themes/theme.schema.json` requires at least one `theme_roles`
entry with `minProperties: 1`. Runtime `ThemeDocument::validate` now enforces
the same rule, so `parse_theme` rejects documents that violate the published
shape. The focused locked/offline regression passed 1/1; evidence:
`target/headstart/evidence/m716-theme-roles-nonempty-test.log`. No broader
workspace or native theme acceptance was run.

## M723 metadata object shape

The published `themes/theme.schema.json` requires `source`, `token_sources`,
and `derivation` to be JSON objects. Runtime `ThemeDocument::validate` now
checks the same shapes; focused regressions reject non-object values for each
field. The locked/offline focused test passed 1/1; evidence:
`target/headstart/evidence/m723-theme-metadata-object-shapes-test.log`. No broader
workspace or native theme acceptance was run.
