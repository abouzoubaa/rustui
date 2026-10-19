# M67: native Shadow modifier

## Public contract

```rust,ignore
Shadow("card-elevation", child)
    .elevation(ShadowElevation::Medium)
    .radius(10.0);

let style = ShadowStyle::new(
    -3.0, 5.0, 8.0, -2.0,
    Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.3 },
)?;
Shadow("explicit-elevation", child).style(style).radius(10.0);
```

`Shadow(id, child)` accepts any RustUI View and defaults to the active upstream
theme's Medium elevation. Give sibling wrappers distinct stable IDs. Shadow's
typed scope is separate from Background and Border, so nested families can reuse
the same ID. Changing style or elevation on the same identity preserves the
retained child's path; normal omission/remount rules still apply.

`ShadowElevation::{Small, Medium, Large}` resolves through the existing active
`gpui_component::Theme::shadow_tokens()` method. This preserves its layer vectors
and its disabled-shadow policy. No new semantic shadow token or palette is
invented. The pinned projection currently derives these elevation presets from
the theme's shadow enable flag and native transparent color; it does not imply
an independently editable RustUI elevation document/schema.

`.style(ShadowStyle)` replaces theme elevation with one explicit native outer
shadow. `.elevation(...)` switches back to the current theme. The last source
selection wins. An explicit style keeps its supplied color and is independent
of the theme's shadow enable flag.

`ShadowStyle::new(offset_x, offset_y, blur, spread, color)` returns a validated
value or `ShadowStyleError`:

- Both offsets must be finite; negative offsets are valid.
- Blur must be finite and nonnegative; zero is valid.
- Spread must be finite; signed values retain native expansion/contraction
  semantics. Upstream Medium/Large presets themselves use negative spread.
- Every Hsla channel must be finite and in `[0, 1]`, matching the native color
  contract and the existing Canvas color-check policy.

Style fields are private, so callers cannot bypass validation with a struct
literal. The Canvas helper is private to that component; Shadow performs its
small local channel predicate rather than coupling a foundational modifier to a
Canvas component or adding a new generic validation abstraction.

`.radius(value)` sets the native box's corner radius. It accepts finite
nonnegative logical pixels and panics for invalid input, consistently with
Background/Border radius builders. Radius does not clip the child.

## Native implementation and boundaries

The wrapper creates a native flex-column box and uses GPUI `BoxShadow::new`,
`.blur_radius`, `.spread_radius` and `Styled::shadow`. It adds no padding, size
constraints, overflow clipping or event handler. Native shadow painting remains
outside layout bounds; it does not reserve space or enlarge the hit target.
Ancestors may still clip shadows according to their existing overflow policies.
Use ordinary padding when the surrounding composition needs visual room.
Offsets are physical x/y and do not automatically reverse in RTL.

This slice exposes outer drop shadows only. It does not implement an inset
shadow API, multiple custom layers, a blur engine, custom GPU rendering, another
renderer or animation runtime. A theme elevation may contain multiple native
layers, which are forwarded unchanged.

Both native box and child lower inside `RetentionScope::Shadow(id)` using the
existing RenderContext. Root integration supplies the enum variant, duplicate
scope detection and native `shadow` identity segment. The actual component owner
continues to own actions, bindings, task/lifecycle state and notifications.
No adapter-owned callback target or retained entity is introduced.

## Source evidence

- `vendor/gpui-pre-0.3.9/src/style.rs`: native BoxShadow fields, constructor and
  blur/spread setters.
- `vendor/gpui-pre-0.3.9/src/window.rs`: `paint_drop_shadows` uses native offset,
  signed spread, blur, color and corner geometry in the existing render path.
- `vendor/gpui-component-0.7.3/src/theme/mod.rs`: public `shadow_tokens` projection
  and disabled policy.
- `vendor/gpui-base-0.7.3/src/theme_tokens.rs`: Small/Medium/Large elevation
  vectors, including negative spread in Medium/Large presets.
- `crates/rustui/src/layout/container.rs`: existing native shadow and radius
  styling; `components/advanced/canvas.rs`: established Hsla channel predicate.

No vendor, dependency, revision, feature or shared runtime file was changed by
this implementation agent. This is reuse of existing native behavior, not an
advanced rendering-phase implementation or a performance claim.

## Gallery and verification

### Bounded native acceptance

The bounded M67 CUA observations are recorded in
`target/headstart/milestone67-shadow-gui.json`. In Dark appearance, Small,
Medium and Large theme shadows were visible. Bound A/B fields sharing the same
inner ID remained independent after switching from theme to explicit style,
changing offsets/blur, and updating the parent. The owner-action Button hit
once; omitting A preserved B, and restoring A recovered its root-bound value.
Large elevation was selectable, and Arabic RTL translated Close while keeping
the edited values.

Light appearance and English LTR restoration were also observed: theme shadows
remained visible, edited A/B values persisted in Light appearance, and
switching back to English restored LTR composition without losing edits. The
bundle has no saved screenshot image. Exact pixels, a native shadow-disable
toggle check, full keyboard and AX speech checks, and performance measurements
remain pending. This is bounded native acceptance, not a pixel, performance,
or full-accessibility claim.

`stories/shadow_gallery.rs` uses public RustUI APIs. It includes:

1. Current-theme Small/Medium/Large samples.
2. Two bound TextFields under distinct Shadow IDs using the identical explicit
   child ID, with independent mirrors. Fixed-size Containers provide geometry
   without a nested Keyed/Component/decoration scope masking Shadow isolation.
3. Theme/explicit source selection, explicit offset/blur changes, semantic
   elevation cycling, parent rerender and omit/restore A.
4. A shadowed owner-action Button composed with Border and Background using the
   same ID across families, plus translated text and logical padding for RTL.
5. Visible outer spacing for shadow paint, without overflow suppression.

Playground's existing appearance controls change the active theme; semantic
samples follow its shadow policy. Explicit samples keep their supplied color.
Edit distinct field values, change source/shape/elevation and update the parent;
mirrors should remain independent. Omit/restore A should preserve its root-bound
string while its native editing state remounts. Confirm the shadowed Button
updates its actual owner, logical padding responds to RTL, and shadows remain
visible where ancestors permit painting. Theme-off behavior is a valid empty
semantic shadow, not a failed explicit style.

Six focused tests cover identity validation, native zero/signed parameter
forwarding, invalid geometry, all color channels, default/source selection and
radius with an unclipped native style, and exact upstream theme vectors and
disabled policy. They are included in the M66–M68 workspace suite, which passed
221 tests across 12 binaries. Root owns companion namespace tests, exports,
registration and serial verification.

The bounded M67 native Dark/Light and Arabic/English scenarios are recorded in
`target/headstart/milestone67-shadow-gui.json`. The M66–M68 build and workspace
suite passed; the M69–M71 all-targets check later exited 0. Exact pixels,
shadow-disable behavior, full keyboard/AX speech and performance remain
unverified. Unit/style tests and gallery presence do not establish those checks.
