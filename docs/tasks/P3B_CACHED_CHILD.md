# P3B: Fixed-bounds retained child cache pilot

`Component(key, initialize).cached_fixed(width, height)` opts a retained child
without incoming props into the pinned GPUI `Entity::cached` API.
`Component(key, initialize).props(props, reconcile).cached_fixed(width, height)`
opts a props-driven retained child into the same cache. Dimensions must be finite,
positive logical pixels and provide the definite layout bounds required by GPUI.
The default `Component` and props-driven paths remain uncached. The cache
modifier must follow `.props(...)`; adding `.props(...)` after
`.cached_fixed(...)` is rejected.

The cache modifier automatically lowers through the normal uncached entity path
when the current typed `EnvironmentValues` scope is nonempty, GPUI reports an
active native drag, or accessibility is active for the window. The typed scope
fallback protects the pilot from RustUI's current environment refresh path,
which replaces a child's environment without notifying its GPUI entity.
Comparing cloned `Rc` identities would detect replacement but not interior
mutation, so it is not used as a general cache-invalidation contract.
`App::has_active_drag()` covers GPUI's active native drag state; external drags
and interactions before drag activation have not been accepted. Drag-capable
views remain outside this pilot until native drag verification is recorded.

The accessibility fallback uses GPUI's `Window::is_a11y_active()`.
`vendor/gpui-pre-0.3.9/src/window/a11y.rs:274-280` clears the AX node builder
each frame, and `vendor/gpui-pre-0.3.9/src/element.rs:426-499` creates child
AX nodes during prepaint. Cached prepaint reuse in
`vendor/gpui-pre-0.3.9/src/window.rs:3996-4045` replays hitboxes and the
dispatch subtree but does not replay AX nodes. Cached paint reuse replays mouse
listeners (`window.rs:4061-4105`). These paths support native hit testing by
source inspection, but a native click check is still required.

For the allowed path, GPUI reuses the entity subtree while bounds, content mask,
and text style match and neither the child entity nor window is dirty
(`vendor/gpui-pre-0.3.9/src/view.rs:524-535`). A full-window refresh
(`window.rs:2375-2380`), including a focus change (`window.rs:2405-2415`),
bypasses reuse. RustUI owner callbacks and task state changes notify their host.
Localization has a host global subscription; the supported theme update path
refreshes windows. These mechanisms are source-level reasons to test caching,
not GUI acceptance.

Focused unit tests cover valid and invalid dimensions, typed-environment,
active-drag, and active-accessibility fallback eligibility for both child paths,
plus rejection of props after the no-props cache modifier. These unit tests are
source-level checks, not GUI or native acceptance. Root-owned native acceptance
must check cache-hit lowering counts, child state/task/props updates, click and
keyboard activation, focus and accessibility, theme/locale/font changes, sizing,
reorder/removal/restoration, and the uncached fallback. A separate drag-capable
story and native drag review are required before claiming drag support. P3A host
diagnostics and a contemporary paired workload baseline must establish any
performance benefit; this pilot adds no upstream patch, revision, dependency,
or second cache/runtime.

## Playground pilot story

`apps/rustui-playground/src/stories/cache_gallery.rs` defines a cached
`CachedLeaf` and an uncached `UncachedLeaf` with the same fixed visible size and
separate local counter actions. A parent-only action changes the gallery owner.
Diagnostic controls enable, clear, sample, and disable per-type host lowering
counts. They refresh only the gallery owner through `AppAction::then` and
`OwnerAction::action`; calling `refresh_windows` during these steps would force
a GPUI cache miss and invalidate the comparison. A sample displays counts from
before its own display refresh, which itself is recorded for the next sample.

This story exercises parent and child notifications. Typed-environment fallback
is covered by the pure eligibility unit test; no environment toggle is shown in
this two-host gallery. Before adding the active AX fallback, three successive
parent-only clicks after clearing counters and sampling showed matching
`CachedLeaf` and `UncachedLeaf` lowering counts of 4. This does not demonstrate
a cache hit or a performance benefit. The AX-assisted interaction exercises the
normal lowering fallback. The story is not a speed benchmark or drag acceptance.

## Controlled no-AX workload

Run `./scripts/cargo-headstart run -p rustui --example cache_benchmark` from
the repository root. This opt-in framework example opens one GPUI window with
RustUI retained children with and without `cached_fixed`. It uses GPUI's
`Window::on_next_frame` scheduler for 10 warmup parent updates, then enables
the existing lowering diagnostics for 100 parent-only updates, one cached-child
state mutation, and one uncached-child state mutation. It records raw per-type
lowering counts, both logical values emitted by the children, cache-hit and
invalidation observations, and an AX-active flag in a new JSON file under
`target/headstart/`. The example exits through `App::quit`; a 30-second GPUI
timer records a timeout result and exits if frames stop progressing.

Any active AX session is recorded as `excluded_ax_active` because the safe
fallback intentionally uses normal lowering then. A completed run with equal
parent-phase counts would also remain evidence of no demonstrated cache hit.
The workload is not a frame, GPU, or presentation benchmark and cannot establish
a speed benefit without a separate paired baseline.

### Executed no-AX results

Three isolated runs completed with `ax_active_seen: false`. Each run recorded
0 cached-child lowerings and 100 uncached-child lowerings during 100 measured
parent updates after 10 warmups. Notifying the cached child produced its own
lowering, and the final logical values emitted by both children were 1. Raw
results are `target/headstart/cache-benchmark-58068-1792309557838561000.json`,
`target/headstart/cache-benchmark-58230-1792309576055094000.json`, and
`target/headstart/cache-benchmark-58329-1792309588158481000.json`; the
aggregate and binary SHA-256 are in
`target/headstart/milestone22-cache-workload-repetitions.json`, with build
provenance in `target/headstart/evidence/milestone22-cache-workload-build.json`.
This demonstrates cache-hit lowering avoidance and local child invalidation in
the controlled no-AX workload for the measured no-props child. It does not
measure a speedup or establish cache behavior for props-driven children.

Theme and locale invalidation, removal/restoration, reordering, and native drag
behavior remain outside this result. They require separate source and native
checks before broader P3B acceptance.

The same example also has an opt-in `--invalidation` path. It steps a separate
cached child through a typed environment scope and fallback, removal and
restoration, an English to French locale change, and Light to Dark to Light
theme activation. Each step records its own child lowering delta, live weak
entity state, mount/unmount counts, logical text, locale generation, and active
theme background color in JSON. The path excludes active AX sessions and uses
the same frame timeout. It does not establish pixel or visual output,
reordering, or drag behavior.

One native `--invalidation` run completed with AX inactive:
`target/headstart/cache-benchmark-65285-1792310422515793000.json`. The typed
scope and fallback each caused one child lowering and emitted `scoped:English`
then `absent:English`. Removal left the weak child handle dead and called
unmount once; restoration created a live child and raised mounts to two. The
locale step caused one lowering, emitted `absent:Francais`, and advanced the
locale generation to one. Dark and Light activation each caused one lowering;
the active theme background lightness changed from 0.9607843 to 0.11764706 and
back. These are logical and theme-global observations, not visual acceptance.

## Native accessibility fallback review

M22 Guard Review confirmed both child buttons appear immediately in the native
AX tree. Clicking the cached-pilot child changed its visible local count from
0 to 1 while the uncached control stayed at 0; both remained in the AX tree.
This exercises normal rendering under active accessibility, not a cache hit.
Evidence: `target/headstart/milestone22-cache-partial-gui.json`. The subsequent
unit/integration run passed 77 tests across 12 binaries
(`target/headstart/milestone22-23-tests.log`). That recorded run predates the props-view cache API and does not verify it. A
separate bounded Playground spot-check recorded in
`target/headstart/milestone-props-cache-gui.json` changed the incoming revision
from 0 to 1 while the child local count stayed at 1, then rerendered the parent
while both values remained unchanged. This verifies props propagation and local
state preservation in the props-driven child; it does not establish a GPUI cache
hit or performance benefit. Keyboard/focus, drag, theme/locale, removal/restoration,
and screen-reader acceptance remain pending. Cache performance acceptance remains pending.
