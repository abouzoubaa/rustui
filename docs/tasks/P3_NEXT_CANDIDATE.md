# P3 next candidate: measure retained child rendering before caching

**Proposal only.** No upstream change, GPUI Fast activation, patch validation,
benchmark run, or cache behavior is authorized by this note.

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

RustUI retains child `Entity<ComponentHost<C>>` values and currently lowers both
`ComponentView` and `ComponentPropsView` with `child.into_any_element()`
(`crates/rustui/src/view.rs:645-670`). The child host performs RustUI lowering
in `ComponentHost::render` (`crates/rustui/src/runtime/component_host.rs:243-282`).
Changed props notify the child, but `refresh_environment` currently replaces
its environment without notifying (`view.rs:339-385`, `component_host.rs:238-240`).
Caching that path as-is could leave environment-driven content stale. GPUI's
cached-view click and focus behavior also needs native verification, regardless
of the pinned code's replay logic.

## Smallest next implementation

Add **opt-in RustUI render diagnostics** around `ComponentHost::render` before
adding a cache modifier. Count host renders and time only RustUI's call to
`component.into_element`; publish bounded, per-owner observations through
RustUI's public API or an explicitly diagnostic Playground story. Keep recording
off by default, avoid a second scheduling/rendering system, and name the metric
`RustUI lowering duration`. It excludes GPUI layout, prepaint, paint, platform
submission, display presentation, and input latency. Measure the diagnostics'
own overhead with the same executable and workload before using the durations
to rank cache candidates. The pinned timing-hook audit in
`docs/tasks/P2C_GPUI_TIMING_HOOKS.md` sets this measurement boundary.

If a retained subtree is a measured hotspot, the follow-up cache experiment is
an opt-in fixed-bounds modifier on an entity-backed `ComponentView`; leave
uncached behavior as the default. Before exposing it, specify how parent props,
typed environment changes, theme/locale changes, child state notifications,
layout/size changes, and unmount invalidate the cache. Use GPUI's existing
`.cached(style)` at the retained-child lowering point; do not copy its cache,
introduce a second runtime, or change vendor source. Require click, keyboard,
focus, accessibility, reorder, and removal/restoration checks on the native
Playground story. A frozen subtree with stale values fails the experiment even
if CPU use falls.

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
