# M64: reusable Background and Border modifiers

## Public API

```rust,ignore
Background("card-fill", child, Surface::Panel).radius(8.0);
Border("card-edge", child)
    .width(2.0)
    .radius(8.0)
    .color(BorderColor::Accent);
```

These modifiers accept arbitrary RustUI views, including text, controls, retained
components and other wrappers. IDs are explicit and mandatory, matching the
ownership needed for reusable wrappers. Keep the ID stable when changing style;
give sibling wrappers in one family different IDs. Background and Border use
different typed namespaces and may reuse the same ID across families.

Background reuses existing `Surface::{Background, Sidebar, Panel}` tokens with
the same active upstream palette mapping as Container:

| Token | Active GPUI Components palette |
|---|---|
| Background | background |
| Sidebar | sidebar |
| Panel | secondary |

Border defaults to one logical pixel, square corners and the active theme border
color. `BorderColor::{Border, Accent, FocusRing}` resolve respectively to current
`border`, `primary` and `ring` palette entries. FocusRing is a decorative color
choice; it neither requests focus nor derives a focus state.

Radius and border width accept finite nonnegative logical-pixel values and panic
for negative/nonfinite input. Zero radius gives square corners; zero border
width draws no stroke. Theme colors resolve during lowering, so changing the
active appearance uses the current palette rather than a cached color snapshot.

## Native layout and ownership

Each wrapper adds one native flex-column layout box, using the same style
facilities as Container: background fill, border width/color and corner radius.
It does not add padding, clipping, event handlers, a renderer or reactive state.
Child sizing and parent flex rules determine the box's dimensions. Explicit
sizing/padding remains available through Frame/Container and existing insets.
A native border participates in box layout; it is not an overlay stroke.
Composition order therefore has ordinary nested-box semantics.

`Background("fill", Border("edge", child), surface)` and the reverse order
are both allowed, but they paint different nested boxes. Corner rounding alone
does not clip descendants. Child foreground/font styling is inherited normally;
the modifiers do not replace it.

Required integration companions are `RetentionScope::Background(Identity)` and
`RetentionScope::Border(Identity)`, host duplicate-scope detection, and distinct
native `background`/`border` scope encodings. Each wrapper lowers both its native
box and child inside that typed scope. It retains the actual component updater,
binding owner and environment; there is no adapter-owned callback target.
Retained controls with the same explicit child ID are isolated across sibling
wrapper IDs and across the two families. Changing only color/width/radius keeps
those child paths stable. Removing a wrapper follows normal retained cleanup;
restoring it creates fresh native editing state while separately owned binding
values can remain.

Frame currently wraps native sizing/alignment without adding its own retained
identity; padding is represented by LogicalInsets/Container. Those APIs were
inspected and remain unchanged. This slice supplies explicit decoration identity
rather than changing their ownership behavior.

## Gallery

`stories/decoration_gallery.rs` uses only public RustUI APIs and includes:

- Translated Text with a semantic background and asymmetric logical padding.
- A bordered Button updating its actual component owner.
- Two Background siblings and two Border siblings with independent root-bound
  TextFields. All four fields intentionally share one explicit child ID, while
  wrapper A/B IDs are reused across the two families.
- Parent rerender, style toggle, omit/restore A and independent value mirrors.
- Light/dark actions through public theme activation for the included macOS27
  theme, plus the existing locale/RTL selector.
- A composed Border/Background example.

The fixed-width Containers around the fields establish geometry only; they do
not add a keyed or retained component scope that could mask namespace defects.
Edit four distinct values, then change style, parent state, theme and locale.
Values must remain independent. The bordered Button must update the owner hit
count. Omit/restore A must keep root-bound strings while resetting omitted native
editing state. Check logical padding after RTL selection. Inspect fill/stroke
colors in light and dark appearances, including the static FocusRing token.

## Verification status

Eight focused unit tests were added: four for Background and four for Border.
They cover identity validation, default/custom native geometry without clipping,
zero and invalid dimensions, and active palette mapping. Root owns the companion
scope/namespace tests, exports, module/story registration and serial
Cargo/native verification.

## M64 executed acceptance

The reviewed native bundle observed four same-ID field values remaining
independent across two Background and two Border scopes. Parent rerender, style
toggle, a bordered-button hit, Light/Dark appearance, Arabic translation/order,
and omission/restoration were exercised; root-bound A values remained while
omitted editor-local state reset and both B fields stayed unchanged. Rounded
semantic fill, thicker accent borders, and composed fill/stroke were visible.
Evidence: `target/headstart/milestone64-decoration-gui.json`; bundle SHA-256
`e917a357fbe391eb428d0cb029d87fecb41cf7aba6e9dc128804618a4dd5d678`.

The M63–M65 workspace suite passed 200 tests, 0 failed across 12 binaries in
`target/headstart/milestone63-65-final-tests.log`. Build succeeded with all
three pinned GPUI dependencies fresh (`target/headstart/milestone63-65-reuse-report.json`),
and the locked/offline all-targets check exited 0
(`target/headstart/milestone63-65-all-targets-check.log`).

Limits: rounding does not clip descendants; FocusRing remains a decorative
color, not actual focus. Exact pixels, layout impact, native undo, full keyboard
behavior, and screen-reader speech were not accepted. No performance claim is
made. No dependencies, upstream revisions, or vendored sources changed.
