# Native Frame constraints and alignment

`Frame(child)` creates a generic layout wrapper around any RustUI view. Unlike
Container's themed box styling, Frame supplies minimum/maximum bounds and
child alignment without adding a theme surface, border, clipping, or state.

```rust
Frame(Text("Content"))
    .width(300.0)
    .height(80.0)
    .min_width(40.0)
    .max_width(200.0)
    .alignment(HorizontalAlignment::Start, VerticalAlignment::Center)
```

This requests width 300 with a native maximum 200; GPUI's native layout clamps
that fixed request. Frame forwards the dimensions rather than precomputing a
second layout result. The API provides `.width`, `.height`, `.min_width`,
`.max_width`, `.min_height`, and `.max_height`, all in logical points. Each
accepts finite nonnegative values, including zero. Negative, NaN, and infinite
values panic. Min/max setters reject contradictory bounds in either order;
equal bounds are valid. A fixed request outside those bounds is valid and is
clamped by native constraints. Changing an existing pair must keep each
intermediate builder configuration valid.

By default, dimensions are automatic and the content holder is centered on
both axes. Frame requests no fill or grow behavior; its parent can stretch an
unconstrained dimension. A finite maximum is only a ceiling, not a request to
fill available space. There is no infinity sentinel or new sizing runtime.
A frame with a fixed dimension disables flex shrinking like the existing
Container fixed-size path; it can overflow a smaller parent. Min/max-only
frames retain native flex shrinking. Frame sets zero native automatic minimums
before applying explicit minima so intrinsic content does not prevent a zero
constraint. Oversized content can overflow; Frame adds no clipping.

## Logical alignment

`.alignment(horizontal, vertical)` reuses the existing `HorizontalAlignment`
and `VerticalAlignment` enums. All their variants are supported:

- Horizontal Start/End follow the active layout direction; Center centers the
  content holder. A normal row explicitly swaps native start/end justification
  for RTL. GPUI's `justify_start` uses absolute Start, so reversing the row alone
  would not establish the logical edge contract.
- Vertical Start/Center/End use native cross-axis alignment. Baseline delegates
  to native `items_baseline`; one child has no peer baseline to align against.
- Horizontal Stretch gives the holder 100% width and zero minimum width;
  vertical Stretch gives it 100% height, zero minimum height, and native stretch
  alignment. These percentages follow native resolution against the frame;
  without a definite dimension they do not invent an intrinsic measurement.

Outside Stretch, a nonflexing content holder preserves the child's own native
size proposal for alignment. Stretch changes the holder's extent; the child
still obeys its own native fixed sizes and constraints.

## Native mapping and ownership

`crates/rustui/src/modifiers/frame.rs` maps directly to GPUI `Styled` dimension
and flex alignment methods. The pinned bridge forwards `size`, `min_size`, and
`max_size` to Taffy (`vendor/gpui-pre-0.3.9/src/taffy.rs:489-491`), whose native
flexbox algorithm clamps style dimensions to the resolved bounds
(`vendor/taffy-0.13.0/src/compute/flexbox.rs:180-200`). There are no dependency,
feature, upstream source, ownership, scheduler, or cache changes. The child is
lowered with the existing owner/context; Frame creates no retained entity.

## Bounded native acceptance

The four source checks inspect native style construction for zero/fixed
constraints, natural defaults, logical alignment, Stretch/Baseline mappings,
invalid values, and contradictory bounds. They inspect forwarded native
styles; they do not run native layout or establish exact pixel positions.

M55 received bounded visual acceptance using native CUA screenshots and the
accessibility tree. Evidence is in
`target/headstart/milestone55-frame-gui.json`; the reviewed application bundle
`target/headstart/verification/M54 M55 Paint Review.app` has SHA-256
`316b268732146400e3f6efecb788317a509f3f11c8b48bff6dd764afd30bea7f`. In LTR,
Start appeared left and End right; the 240 requested width with 160 maximum
appeared narrower; and the zero-width sample rendered without a crash. The
button inside a fixed Frame changed from count 0 to 1 and retained count 1 after
a runtime locale switch. Under Arabic, Start appeared right and End left while
centered content remained centered.

The shared M54–M55 paint test log reports 161 passed and 0 failed across 12
binaries (`target/headstart/milestone54-55-paint-tests.log`). This is the earlier
M55 result; M74 below later exercised Stretch and Baseline. Full keyboard and
screen-reader acceptance remains pending. These bounded reviews do not claim
pixel measurements or complete accessibility acceptance.

## M74 bounded native acceptance

The final M74 Frame Review bundle (`target/headstart/verification/M74 Frame Final Review.app`, SHA-256 `a993fc7eeb1f1749d190c92635bbb59a28fa6501ab1351153053ecdcd266eb58`) verified these interactions:

- Center retained the typed Frame value. Stretch expanded the background to the
  frame bounds while retaining that value.
- Baseline rendered without a crash and retained the text `Frame retained!`.
  Command-Z restored the prior text after the mode change.
- Cycling back to Center retained the value; clicking the framed button changed
  its count from 0 to 1.

Initial tiny-editor and width-160-only attempts failed because the Input holder
collapsed. Adding a stable inner `Frame` width of 160 with horizontal Stretch
resolved percent-width layout; the outer alignment cycle stayed unchanged. The
bounded record is `target/headstart/milestone74-frame-gui.json`; the build log
`target/headstart/milestone74-inner-stretch-build.log` exited 0. The final M74
workspace suite passed 238 tests, 0 failed across 12 binaries
(`target/headstart/milestone74-workspace-tests.log`, Cargo exit 0), and the
locked/offline all-targets check exited 0
(`target/headstart/milestone74-all-targets-check.log`).

This was a single-child example, so no peer-baseline placement is established.
No exact-pixel, caret/selection, full keyboard/accessibility, RTL, or
cross-platform acceptance is claimed. Private identity lifetime counters were
not added; retained Undo provides bounded editing-state evidence only.
