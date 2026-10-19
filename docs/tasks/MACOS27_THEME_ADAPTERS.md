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

The remaining source-only selectors were checked against the current public
RustUI builders, renderer state, and the `macos27.json` rule predicates. No
renderer-only changes were made because each item requires a semantic input
that the current API or GPUI wrapper does not expose.

- **Text and Label vibrant material:** `TextView`
  (`crates/rustui/src/components/basic/text.rs`) and `LabelView`
  (`crates/rustui/src/components/basic/label.rs`) expose text, typography,
  and label content but no material setting or inherited material
  environment. `components/theme.rs::selector_for_mode` always supplies
  `material = opaque`; the Text and Label renderers also name opaque parts
  directly. Changing these to vibrant unconditionally would apply a material
  variant to ordinary text on opaque surfaces. Supporting this correctly
  requires an explicit or inherited material contract and renderer
  consumption.
- **Icon clear/pressed:** `IconView` exposes name, size, accessibility label,
  and identity; it is not interactive and carries no pressed state or semantic
  part. The `icon` rule is the only rule it resolves. The `clear` parts in the
  theme belong to a clear control and have `interaction = rest|pressed`; using
  them for every named icon would conflate icons with a stateful clear button.
  A clear-control API and renderer must own that state before those rules are
  consumable.
- **Image utility images:** `ImageView` exposes source, fit, explicit size,
  and accessibility metadata. It resolves the generic `image` part. The
  separate `images` rules require `chrome = utility`; there is no image role,
  utility context, or chrome selector in the public builder or render
  context. Selecting utility chrome for every image would be an unsupported
  semantic assumption. A typed image role or inherited chrome context is
  needed to select the utility rules selectively.
- **TextField `control_search_field`:** `SearchField` creates the existing
  `TextFieldKind::Search`, but the renderer delegates interaction to GPUI
  Components' generic `Input` and resolves `search_field/list_search`.
  RustUI tracks disabled state and bound text, but the wrapper does not expose
  focused/rest transitions from `InputState` to the theme selector. The
  `control_search_field` rules distinguish disabled, focused, and rest states
  and include vector artwork/cursor properties. Mapping only static
  appearance would leave state-specific behavior unimplemented; applying the
  full rule requires a deliberate search-control adapter that receives those
  states and supports the declared artwork properties.

These are framework integration gaps rather than Playground configuration
gaps. Playground can verify the framework after these contracts are designed,
but a Playground-only style override would conceal rather than resolve them.
