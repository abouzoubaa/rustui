# M68: themed Card layout primitive

`Card(id, child)` provides a visual panel surface around any RustUI `View`. It
uses the active GPUI Components theme's Panel surface, applies 16 logical pixels
of padding on every side, a 10 logical pixel radius, and the theme's Medium
shadow elevation by default. The caller supplied ID scopes the child retention
namespace. A Card-specific scope keeps that namespace separate from a sibling
`Shadow` that happens to use the same ID.

The builder methods are:

- `.padding(value)` sets equal padding in logical pixels.
- `.radius(value)` sets the corner radius in logical pixels.
- `.elevation(ShadowElevation::{Small, Medium, Large})` selects a current-theme
  elevation.
- `.border(enabled)` toggles the one-pixel active-theme border; it defaults off.

Padding and radius accept finite nonnegative values, including zero, and panic
for negative or nonfinite values. Radius rounds the card surface and shadow box;
it does not clip the child. Card adds no focus behavior, accessibility role,
action handling, or component owner. The active theme supplies the Panel color,
border color, and shadow vectors; the Card does not define a separate palette.

`apps/rustui-playground/src/stories/card_gallery.rs` demonstrates default and
custom styles, light and dark theme changes, locale direction changes, two
bound inputs with the same local child ID under distinct Card IDs, and omission
and restoration of one card while the other input remains editable.

## Acceptance

1. Confirm default cards use Panel fill, 16 px padding, 10 px radius, and Medium
   theme elevation. Compare Light and Dark appearances.
2. Toggle custom styles and border; confirm spacing, radius, elevation, and
   border update without changing either input value.
3. Change the locale direction and confirm the cards remain usable and follow
   the existing logical layout direction.
4. Edit both fields despite their shared child ID. Omit and restore the first
   card, then confirm the second field remains independently editable and both
   values follow the documented remount behavior.
5. Confirm zero padding/radius are accepted and negative or nonfinite values
   panic with the documented validation messages.

## Bounded native acceptance

The M68 CUA observations are recorded in
`target/headstart/milestone68-card-gui.json` against corrected bundle SHA-256
`c19a89cd2fe3597cf1cd487bcf352486afb6debc3c8c036c53a28a0c1661fe25`.
Both Cards used an explicit equal width of 260 logical pixels in the gallery;
both editable controls were usable and same-ID child values remained
independent through custom styles, parent updates, omission/restoration, Light
appearance, Arabic RTL, and English LTR restoration.

The equal width is a gallery layout choice, not an intrinsic Card sizing rule.
Exact pixel/padding measurement, cursor/undo, full AX speech/keyboard, and
performance acceptance remain unverified. Compilation and unit tests do not
establish those native behaviors.

## Verification status

Seven focused unit tests cover identity validation, default and custom themed
composition, padding and radius validation, elevation selection, and border
configuration. The M66–M68 workspace suite passed 221 tests across 12 binaries;
see `target/headstart/milestone66-68-tests.log`. The build succeeded with all
three pinned GPUI dependencies fresh; see
`target/headstart/milestone66-68-reuse-report.json`. Bounded native acceptance
is recorded above; the listed visual, input and accessibility limits remain.
