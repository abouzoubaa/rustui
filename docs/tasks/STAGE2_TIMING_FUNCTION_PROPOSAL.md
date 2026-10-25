# Stage 2 timing function API proposal

**Status (2026-10-25):** The bounded API below is implemented under the human's
sole-session roadmap-completion authorization. Original authority review found
only scaffold/proposal wording for its file/API gate. This implementation uses
the existing GPUI easing hook, adds no dependency or scheduler, and does not
change the runtime strategy. Curves, linear default, retained mount-time curve,
and duration/frame-limit forwarding have focused source tests; native motion
and reduced-motion acceptance remain open.

## Existing gate and boundary

`crates/rustui/src/animation/timing.rs` is a planned scaffold. Its module
comment says it has no runtime behavior or public type contract and requires a
separate proposal and explicit approval. `docs/FILE_REGISTRY.md` classifies
the file as deferred pending an approved API and upstream integration.

The pinned GPUI snapshot already supplies the needed runtime: `Animation::new`
creates a one-shot linear animation; `Animation::with_easing` accepts a
`'static` progress function; `AnimationExt::with_animation` owns frame
scheduling and honors the app's reduced-motion setting. RustUI's existing
`AnimatedOpacity` already delegates to those GPUI APIs and preserves its
stable animation identity and optional frame limit.

## Proposed public contract

Add a public `TimingFunction` enum in `rustui::animation::timing` with four
standard, non-overshooting curves:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimingFunction {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl TimingFunction {
    pub fn sample(self, progress: f32) -> f32;
}
```

`sample` maps normalized progress in `0..=1` to normalized progress in
`0..=1`, preserving both endpoints. Finite values outside that interval are
clamped to the nearest endpoint; NaN propagates. The curves are:

- `Linear`: `t`
- `EaseIn`: `t³`
- `EaseOut`: `1 - (1 - t)³`
- `EaseInOut`: `4t³` below `0.5`; `1 - (-2t + 2)³ / 2` at and above `0.5`

Add `AnimatedOpacity::timing_function(TimingFunction)`, defaulting to
`Linear` so existing animations retain their current behavior. The builder
selects a curve before mounting; the animation remains one-shot and mount
only, and changing its target or curve does not restart it under the same
identity. The internal adapter passes `timing.sample(progress)` to GPUI's
`Animation::with_easing`; it does not schedule frames or manage animation
state itself.

## Minimal implementation files

1. `crates/rustui/src/animation/timing.rs` — enum, `sample`, and focused unit
   tests. `animation/mod.rs` already publicly declares the module.
2. `crates/rustui/src/modifiers/opacity.rs` — store the selected function,
   add the builder, and pass its easing closure to the existing GPUI
   `Animation`.
3. `apps/rustui-playground/src/stories/animation_gallery.rs` — show the four
   curves using the existing restart control and `AnimatedOpacity` public
   API; keep the current linear example as the compatibility case.

No manifest, vendor, runtime, or shared roadmap/ledger change is proposed.

## Non-goals

- No animation scheduler, clock, frame loop, or second reactive/rendering
  runtime; GPUI remains the sole owner of scheduling and reduced motion.
- No springs, cubic-Bezier parameters, arbitrary closures, delays, repeats,
  keyframes, view transitions, or animation retargeting.
- No changes to `timing.rs` consumers outside `AnimatedOpacity`; the separate
  `transition.rs` and `animator.rs` gates remain open.
- No claim of native visual, screen-reader, frame-rate, or performance
  acceptance from unit tests or a successful compile.

## Focused acceptance proposed

- Unit tests check every curve's endpoints, representative values, monotonicity,
  and bounded output for a deterministic set of inputs across `0..=1`; test
  endpoint clamping and NaN propagation separately.
- `AnimatedOpacity` tests show that the default remains linear and that
  duration, one-shot behavior, optional `max_fps`, and GPUI easing selection
  are all forwarded together.
- The Playground story compiles and presents all four curves with the same
  duration and restart semantics. Any native motion/reduced-motion observation
  is recorded separately from source-level acceptance.

## Original proposal decisions (historical)

Please approve or revise:

1. The four-variant curve set and the formulas/clamping behavior above.
2. Adding `AnimatedOpacity::timing_function` as the first public consumer of
   this API, with `Linear` as its default.
3. The three-file implementation boundary and focused acceptance scope.
