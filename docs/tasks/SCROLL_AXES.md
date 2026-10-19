# M62: ScrollView axes and upstream overlay scrollbars

## Public contract

```rust,ignore
ScrollView(content)
    .id("bounded-canvas")
    .axis(ScrollAxis::Both)
    .width(520.0)
    .height(200.0)
    .scrollbars(true)
```

`ScrollAxis` has `Vertical` (default), `Horizontal` and `Both`. Existing
`ScrollView(content).id(...).height(...)` remains supported. The default fills
available width/height, uses vertical scrolling, and draws no scrollbar.
Explicit height retains the existing native `flex_grow_0` behavior. Width is a
new finite positive logical-pixel request. Both dimension builders panic for
zero, negative or nonfinite input; parent flex rules still apply.

Scrollable dimensions require bounded parent space or explicit dimensions.
These builders do not make an unbounded parent intrinsically measurable. Content
is nonshrinking along scrolling dimensions; this is not virtualization.

`.scrollbars(true)` enables the pinned upstream overlay scrollbar. Visibility,
fade behavior and appearance use the current upstream theme policy; enabled does
not mean always visible. Scrollbars do not reserve layout space. No new public
GPUI type, dependency or custom scrollbar implementation is introduced.

## Native ownership and structure

Each typed `RetentionScope::ScrollView` retains one GPUI entity containing a
native `ScrollHandle` and the last axis. Outer root, viewport, content and
scrollbar-overlay IDs remain stable whether bars are enabled or disabled.
The tracked viewport uses native `track_scroll`, `overflow_y_scroll`,
`overflow_x_scroll` or `overflow_scroll`. The upstream `Scrollbar` is a sibling
overlay outside the scrolled content, using the same handle and resolved overlay
bounds. Creating or removing a bar does not replace the viewport or handle.

Content sizing follows the upstream Scrollable adapter's axis contract:

- Vertical: nonshrinking column, full viewport width, natural content height.
- Horizontal: nonshrinking column, natural width (at least viewport width), full
  viewport height. Child HStack/layout still determines content arrangement.
- Both: nonshrinking natural dimensions, at least viewport size.

Ordinary owner rerenders, bar toggles and size changes reuse the native handle.
Native prepaint clamps offsets after content or bounds change. On axis changes,
only newly disabled coordinates are cleared; enabled coordinates remain subject
to native clamping. There is no unconditional offset write or notification.
Omitting a viewport from retained composition releases its state. Restoring it
creates fresh scroll state; separately owned bound values may persist.

The typed scope also contains retained child identities and native child IDs.
An internal typed `ScrollViewContent` scope separates user child keys from the
adapter handle entry, so even a child named `scroll-state` cannot collide with
internal state.
Sibling scroll views can reuse child control IDs while keeping independent
native editing state. Duplicate sibling viewport IDs in the same retained scope
are rejected. Existing Keyboard, GeometryReader and animation scopes remain
separate and unchanged.

## Wheel, nested scrolling and locale

Single-axis viewports use upstream `lock_scroll_axis` (native gesture-axis
restriction on supported platforms). This prevents GPUI's unrestricted
single-axis remapping of an orthogonal delta. It is an explicit wheel-policy
addition to the earlier vertical-only baseline. The upstream helper is a no-op
on wasm, matching its existing platform behavior.

Both enables both physical axes using GPUI's default dominant-axis-per-delta
policy. It does not enable simultaneous diagonal deltas or implement a custom
wheel engine. Nested event routing, edge propagation and scrollbar dragging
remain native behavior and require native acceptance checks.

Content inherits normal environment, locale and layout direction. Physical
horizontal offsets are not reversed for RTL, and an RTL-specific initial scroll
edge is not promised. Locale selection must be checked with both content layout
and wheel behavior visible.

## Source facilities

- `vendor/gpui-pre-0.3.9/src/elements/div.rs`: native overflow, tracked handles,
  input deltas, offset clamping and `ScrollHandle`.
- `vendor/gpui-component-0.7.3/src/scroll/scrollable.rs`: upstream axis sizing and
  sibling scrollbar overlay pattern.
- `vendor/gpui-base-0.7.3/src/scrollbar.rs`: public upstream Scrollbar.
- `vendor/gpui-base-0.7.3/src/event.rs`: upstream gesture-axis helper.

The existing callers in Playground `lib.rs` are sidebar, preview and references.
Their axis remains vertical without call-site changes. No Canvas source changed.

## Gallery and acceptance

`stories/scroll_gallery.rs` has explicit bounded vertical, horizontal and plane
panes, an orthogonal nested viewport, oversized content, bar toggle, parent
update, external content shrink/expand, plane-axis cycling and hide/restore.
Each pane has a root-bound text field and root owner action. The fields
intentionally share an explicit child ID across separate viewport scopes.
The existing locale selector checks inherited child direction/translation.

Native exercise:

1. Move each pane independently. Toggle bars and update the owner; positions
   should persist and bound mirrors should remain independent.
2. Edit fields and invoke their owner actions. Confirm root mirrors/hit count.
3. Scroll near content ends, shrink externally, then expand. Native offsets
   should clamp without jumping back to the prior out-of-range position.
4. Cycle plane axes. Disabled coordinates should clear; enabled ones should
   remain valid after layout.
5. Exercise nested orthogonal gestures and edge routing in both locale
   directions. Do not infer edge propagation solely from model tests.
6. Hide/restore panes. Native positions restart while root strings remain.
7. Show/drag the native scrollbar thumbs and confirm they stay at viewport edges.

Seven source/model tests were added: default/dimension validation, native overflow
and wheel policy, axis-specific content styles, tracked-handle coordinate
retention, native namespace isolation, internal/user key separation and duplicate
retained-scope rejection.
The implementation agent ran rustfmt only. Cargo/build/native GUI checks remain
root-owned and pending; model tests do not establish native scroll acceptance.
No performance or optimization claim is made.

## Opt-in native diagnosis

A native review observed outer-preview movement without confirmed local pane
movement. No layout repair has been accepted from that observation alone.
Launch the packaged app with `RUSTUI_SCROLL_DIAGNOSTICS=1` to log viewport wheel
position/raw delta and native handle offset, maximum offset, viewport bounds and
first immediate child bounds. A post-children root-prepaint snapshot reports
layout/clamped state. `content_size_derived` is viewport size plus maximum offset,
not an independently measured descendant union.

Diagnostics add no notifications, frame requests, timers or offset writes.
The wheel-handler snapshot may precede native default handling; compare it with
the post-children prepaint record. Native GPUI calculates scroll extents from
immediate child layout bounds, so the first content wrapper bounds are relevant
when descendants appear larger than the tracked extent. Root must isolate
coordinates, event delivery and extents before accepting a repair.

## Both-axis intrinsic-height repair

The exact pre-repair record in
`target/headstart/milestone62-native-diagnostics.log` reported a 520 × 200 plane
viewport, an immediate content wrapper of 1100 × 200, and maximum offset
`(580, 0)`, although the gallery's fixed content was 1100 × 560. Native extent
calculation considers immediate child bounds; it could not scroll vertically
when the wrapper reported only 200 pixels of height.

Both now explicitly uses native `items_start` on its row viewport. Default
cross-axis stretch had constrained the auto-height content wrapper to viewport
height. Start alignment lets that nonshrinking wrapper retain intrinsic content
height, with its existing minimum viewport height. No wheel routing, overflow
policy, retained scope or handle identity changed. A regression style test
checks row layout, start alignment, auto content height, minimum viewport height
and zero shrink. Native review later confirmed the repaired Both-axis content
extent and maximum offset; see the M62 evidence below.

The same exact log established horizontal content 1468 × 150 with maximum
horizontal offset 948. The isolated CUA right action emitted native delta
`(0, 0)`, so it cannot verify horizontal wheel scrolling; scrollbar dragging or
an actual nonzero horizontal event is needed. Root also observed vertical inner
rows moving alongside the outer preview, confirming native ancestor propagation.
That propagation policy is preserved rather than silently contained.

## M62 bounded native acceptance

The final native review is recorded in
`target/headstart/milestone62-scroll-gui.json`, bundle SHA-256
`8ead905f2c20ddc003372a7a509627f54df761fd38f622765bcde80412d6f8ba`. Both-axis
content measured 1100 × 560 with maximum offset (580, 360), including the
repaired maximum height for its 1100 × 560 content. Vertical wheel input moved
the Both pane and visible rows; the ancestor preview also received that native
gesture. Dragging the Both pane's horizontal scrollbar moved x to -539.1549
while y remained -131. Toggling bars and updating the parent preserved the
offset. Switching axes cleared newly disabled coordinates and preserved enabled
ones. Shrinking content while y was -41 clamped both offset and maximum to zero;
expanding restored extents. Bound text survived pane removal/restoration, and
Arabic translation plus reversed logical child order were observed.

This is bounded native acceptance, not full-platform acceptance. The CUA
horizontal wheel input generated delta zero, so real horizontal trackpad
movement, standalone horizontal-only scrolling, and nested horizontal gestures
remain unverified. Diagonal concurrent scrolling, shell sidebar/reference-pane
wheel regressions, full keyboard coverage, and screen-reader acceptance also
remain pending. Native nested listeners still propagate to ancestors; no
containment policy was added. Preserve that upstream behavior. No performance
claim is made.

The repaired M61/M62 suite passed 191 tests with 0 failures across 12 binaries
in `target/headstart/milestone62-sizing-tests.log`. The build passed and all
three pinned dependencies were fresh in
`target/headstart/milestone62-final-reuse-report.json`; the final all-targets
check passed in `target/headstart/milestone61-62-final-all-targets-check.log`.
