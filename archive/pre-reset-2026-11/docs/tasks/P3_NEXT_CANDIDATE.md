# P3 next candidate: retained child cache correctness and calibration

**Status:** RustUI host-lowering diagnostics are already implemented and accepted
as P3A. Their implementation is in
`crates/rustui/src/runtime/render_diagnostics.rs`,
`crates/rustui/src/runtime/component_host.rs`, and
`crates/rustui/src/runtime/events.rs`; the public diagnostics surface and
acceptance record are documented in `docs/tasks/P3A_HOST_DIAGNOSTICS.md`.
Native verification is recorded in
`target/headstart/milestone20-host-diagnostics-gui.json`, with the accepted
milestone summarized in `docs/IMPLEMENTATION_ROADMAP.md`. Recording is opt-in
and times only RustUI's `component.into_element` call inside
`ComponentHost::render`.

The next open P3B cache-pilot correctness gate is native keyboard/focus behavior after an
AX-inactive cache hit. P3 diagnostics overhead now has bounded process-level
captures: the older five-pair result is in
`docs/tasks/P3_DIAGNOSTICS_OVERHEAD_RESULTS.md`, and the fresh current-source
32-leaf run is recorded in `docs/IMPLEMENTATION_ROADMAP.md`. Its small CPU
difference crosses 1.0 and is inconclusive. Neither capture establishes cache
benefit or general diagnostics overhead. No cache promotion, upstream change,
GPUI Fast activation, or vendor change follows from these results.

## What the pinned stack already supports

GPUI `Entity<T>::cached(StyleRefinement)` exists in the approved
`gpui-pre 0.3.9` snapshot (`vendor/gpui-pre-0.3.9/src/view.rs:242-251`). It
reuses an entity's rendered subtree until notification, changed bounds or text
style, or window refresh. Its caller must supply definite layout bounds because
GPUI does not measure cached contents. The cache implementation replays
prepaint, dispatch-tree, input-handler, and mouse-listener state
(`view.rs:515-566`, `window.rs:3996-4025`, `window.rs:4062-4091`). This makes an
upstream patch unnecessary for a *candidate* retained-child cache; it does not
prove RustUI interaction correctness.

RustUI retains child `Entity<ComponentHost<C>>` values. `ComponentView` and
`ComponentPropsView` now expose opt-in `cached_fixed` lowering in
`crates/rustui/src/view.rs`; props-changed frames lower normally to preserve
same-frame freshness. A nonempty typed environment scope, active native drag,
or active accessibility session also takes the normal uncached path. The
environment refresh path replaces values without notifying the child host, so
the typed-scope fallback remains part of this pilot's correctness boundary.
Existing native evidence covers cache-hit counts and child state invalidation,
props freshness, environment/theme/locale logical changes, removal/restoration,
and the active-AX fallback. It does not yet verify keyboard/focus replay during
an AX-inactive cache hit (`docs/tasks/P3B_CACHED_CHILD.md`).

## Smallest next acceptance unit

Reuse the existing `apps/rustui-playground/src/stories/cache_gallery.rs` for a
bounded native keyboard/focus probe. With accessibility inactive, establish a
cache hit after a parent-only rerender, Tab to the cached child button, and
activate it with Space or Enter. Verify one local-state increment, move focus
away and back, and repeat. Record accessibility state for each step; if it
becomes active, classify that step as the uncached fallback rather than cache
replay. Preserve the binary identity, key sequence, before/after child state,
and raw observations in the cache-pilot evidence record. Do not use an
accessibility-mediated interaction for the cache-hit portion.

The bounded diagnostics-overhead calibration is recorded in the result file
and the latest current-source run in the roadmap. Do not use their lowering
durations to rank cache candidates: nested host timings overlap, and the runs
provide no cached-versus-uncached comparison. Any broader calibration needs
its own workload and contemporary paired captures. The pinned timing-hook
audit in `docs/tasks/P2C_GPUI_TIMING_HOOKS.md` sets the measurement boundary;
P3A's functional verification alone is not performance evidence.

The fixed-bounds cache modifier is already present for both retained child
paths; uncached behavior remains the default. Use GPUI's existing
`.cached(style)` at the retained-child lowering point; do not copy its cache,
introduce a second runtime, or change vendor source. P3B remains a pilot: do not
promote caching based on the existing lowering-count results or one favorable
performance sample. Broader click, accessibility, reorder,
removal/restoration, and drag checks remain governed by
`docs/tasks/P3B_CACHED_CHILD.md`. A frozen subtree with stale values fails the
experiment even if CPU use falls.

## Evidence and decision gate

The current W1 baseline has native visual acceptance and three fresh launches
of 30 cycles. Gross process CPU median is 1.137 seconds per cycle; resident
memory is about 124 MiB (`docs/tasks/P2E_W1_ACCEPTANCE_BASELINE.md:28-74`).
It measures neither RustUI lowering nor frame presentation and is not evidence
that child caching will help. Reuse its compact W1 actions, reset, process
collector, and a **contemporary** unchanged baseline for the diagnostic and
later cache trial. Record source/binary identity, raw per-run counts and timing,
collector overhead, CPU/RSS, visual outcome, and variance. Compare at least the
existing three launches and 30 cycles per launch; preserve failed runs. Treat
frame, GPU, and main-thread timing as unavailable until validated separately.
Do not adopt a cache on a single favorable sample or an unapproved threshold.

The isolated P3 candidate pipeline in `docs/tasks/P3_PATCH_VALIDATION_DESIGN.md`
is for separately approved upstream revision/patch candidates. It still needs
candidate-only source and target directories, immutable provenance, ordered
patch validation, correctness, native GUI review, paired performance evidence,
and a separate promotion decision. A RustUI-only diagnostic or cache pilot does
not need an upstream patch pipeline; any later upstream candidate must pass it
without altering the accepted vendor snapshot.
