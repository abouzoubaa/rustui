# M58: GeometryReader

## Public contract

`GeometryReader::new(id, |size: GeometrySize| child)` renders a RustUI child
using native resolved container width and height in logical pixels. The factory
is `Fn + 'static`; no borrowed RenderContext/window/app escapes into it.

The default reader fills available width and height. `.width(value)` and
`.height(value)` specify fixed logical-pixel dimensions and reject negative or
nonfinite values. Zero is valid. Contents cannot determine the reader's
intrinsic dimensions. A bounded parent or explicit dimensions must be chosen
for the surrounding layout; a reader cannot guarantee useful dimensions in an
unbounded or otherwise ambiguous parent. A fixed dimension disables flex shrink
for the reader, consistently with existing fixed-size layout adapters.

```rust,ignore
GeometryReader::new("responsive-card", |size| {
    Text(if size.width < 420.0 { "Compact" } else { "Expanded" })
})
.height(240.0)
```

This slice exposes size only. It does not expose global/local coordinates,
observation bindings, intrinsic measurement or a notification loop. The factory
may run repeatedly at identical dimensions and should not mutate owners or
perform observable side effects. State updates belong in normal actions,
bindings and retained child components.

## Native lifecycle and ownership

The pinned `gpui::container_query` requests layout without children. In prepaint
it receives resolved bounds, calls its one-shot factory, lays out that child
with definite available dimensions, prepaints at the container origin, then
paints the stored child. Reader uses this native lifecycle without a timer,
size-notification feedback loop or separate reactive runtime.

A dedicated `RetentionScope::GeometryReader` gives sibling readers distinct,
typed native and retained namespaces and detects duplicate reader identities.
Each reader retains one GPUI entity containing:

- The current owned `RenderSeed`.
- A generation for callback freshness.
- A dedicated standard `RetainedEntities` store for its content.

Normal owner lowering refreshes the seed/generation and captures the current
factory. The native callback holds only a weak reader entity; expired/stale
callbacks return an empty element without invoking the factory or reconciling
content. The current callback reconciles the reader's store using the existing
`begin_frame`/`finish_frame` methods. Stable identities keep child entities;
omitted branches release them. This reconciliation occurs when the native
callback runs, rather than eagerly during owner lowering. Removing the reader
releases its entire store through ordinary parent reconciliation.

`RenderSeed::geometry_context` preserves the actual component updater, owner
ID/type, task owner, environment, control font size, button theme and locale.
The reader entity provides a distinct text namespace, not a replacement action
or binding owner. Its text prefix is separate from LazyGrid cell prefixes.
There are no invented cell IDs. Reader does not invoke root mutations or
notifications from its native callback. Child actions run on later native
input events, using their real owner. Inherited direction remains available to
normal child layouts.

Existing Keyboard and LazyGrid scopes are preserved. No vendor code, dependency
revision, feature flag or public GPUI dependency was added.

## Playground exercise

The public-only `stories/geometry_gallery.rs` includes:

1. Explicit 280/540 logical-pixel width toggle and 240-pixel height.
2. A persistent nested counter, plus a wide-only counter whose branch is omitted
   below 420 pixels.
3. A text field bound to root state, an external root replacement button, and a
   deferred root callback button.
4. Remove/reinsert controls and manually sampled mount/release counters.
5. The existing English/French/Arabic selector and translated child text for
   inherited locale/direction checks.

Increment persistent and wide-only counters, toggle narrow/wide, then sample
lifetimes. Persistent count should survive; the wide branch should remount at
zero. Remove/reinsert should reset both nested owners. Input and root callback
must update root state; external replacement must reach the deferred text field.
Lifetime sampling is explicit: the gallery does not notify during layout.

## Verification status

Seven source/model tests were added: dimensions and zero handling; factory size
forwarding and parent-sized defaults; generation comparison; text namespaces;
native typed namespaces; duplicate reader rejection alongside Keyboard/LazyGrid;
and dedicated-store branch reconciliation.

The repaired Headstart suite passed 180 tests with 0 failures across 12
binaries in `target/headstart/milestone57-59-repairs-tests.log`. The build passed
in `target/headstart/milestone57-59-repairs-build.log`; the reuse report confirms
`gpui-component 0.7.3`, `gpui-pre 0.3.9`, and `gpui-pre-platform 0.3.9` were all
fresh in `target/headstart/milestone57-59-repairs-reuse-report.json`.

The final bounded native GUI run is recorded in
`target/headstart/milestone58-geometry-gui.json`, bundle SHA-256
`b71b44e00a76a4771eaeda22e0f89767c3d46950ca6250805caeb3f0e6346b39`. At the
tested 540×240 size, the input was readable across the width; root callbacks,
typing, external text replacement, and Arabic translation worked. Persistent
state survived switching to 280×240; the wide-only branch released and remounted
at zero. Removing/reinserting the reader released/remounted child state while
preserving root-bound text. This closes the narrow-input repair for these tested
cases only.

The initial narrow-input failure remains recorded in
`target/headstart/milestone58-geometry-initial-gui.json`, bundle SHA-256
`639a9799d484394a7deee59a98e2cb896a4c151bd0717fe83e846997ff0084e5`.

GUI evidence covers only explicit widths 280 and 540 and height 240. Intrinsic,
unbounded, and zero-size GUI behavior, exact pixel geometry, stale callback
rejection, sibling reader isolation, theme/environment/task propagation,
keyboard/screen-reader behavior, and performance were not verified. The model
generation test does not exercise a live stale native callback. No rendering or
performance gain is claimed.

## Native layout correction after review

Initial native review resolved the reader itself to 540 × 240 logical pixels,
but its returned auto-width VStack did not establish a definite width for the
percentage-sized input. The text field appeared narrow despite the correct
reported reader size. Native `layout_as_root(AvailableSpace::Definite(...))`
supplies available space; it does not assign an auto-width root that width.

The reader now wraps generated content in a native flex-column containing
block with its resolved width and height and cross-axis stretch. This lets
auto-width roots and percentage-sized descendants resolve against the actual
reader width. Explicit child width and align-self continue to follow native
flex rules. No overflow clipping or child size override was introduced.
This is a reader implementation correction; the gallery does not force its
VStack to fill as a workaround.

A focused native-style regression covers both gallery sizes and zero size,
checking fixed containing-block dimensions, flex direction, stretch alignment
and unchanged overflow defaults. The correction resolved the narrow input in
the bounded native cases above; those cases do not establish behavior for
intrinsic, unbounded, or zero dimensions.
