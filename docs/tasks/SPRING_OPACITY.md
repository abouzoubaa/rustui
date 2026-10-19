# M57: SpringOpacity

## Contract

`SpringOpacity::new(id, child, target) -> Result<SpringOpacity<_>, SpringOpacityError>`
uses the pinned GPUI `with_spring` implementation. Stable native identity keeps
position and velocity as the target changes, including interruption while moving.
First mount starts at the target. Optional `.from(initial)?` seeds a new identity
only; changing it does not replay an existing spring. IDs must be unique among
siblings in the native composition scope.

GPUI owns animation state, scheduling, settling and app-level reduced motion.
There is no RustUI timer or second animation runtime. Reduced motion resolves to
the current target when GPUI renders the spring. Opacity does not change hit
testing or accessibility semantics.

`SpringConfig::new(stiffness, damping, mass)?` requires positive finite values
and checks native canonical/propagator intermediates for overflow/underflow.
Zero damping is rejected because a displaced spring would not naturally settle.
The default is `(170, 26, 1)`. `.epsilon(tolerance)?` accepts finite
`[f32::EPSILON, 1]`, default `0.001`; native applies this positional tolerance
and scales its velocity tolerance by the natural frequency. The lower bound
avoids an underflowed velocity threshold with valid low-frequency physics.
This is a settling threshold, not a presentation frame-rate setting.

Targets and initial opacity must be finite in `[0, 1]`. The internal spring may
overshoot; presented opacity is clamped to `[0, 1]`. Unexpected nonfinite native
output presents zero opacity defensively. Clamping can flatten visible motion
at a boundary while native motion continues.

## Examples

```rust,ignore
let target = if expanded { 1.0 } else { 0.15 };
SpringOpacity::new("details-opacity", Text("Details"), target)?
    .config(SpringConfig::new(170.0, 8.0, 1.0)?)
    .epsilon(0.001)?;

// Animate only when this identity is first mounted:
SpringOpacity::new("initial-fade", Text("Welcome"), 1.0)?.from(0.0)?;
```

The public-only Playground sample in `stories/spring_gallery.rs` provides target
retargeting, explicit app-level reduced-motion controls, and a child button with
an activity counter. It keeps one stable spring identity.

## Evidence and remaining limits

Source basis: `vendor/gpui-pre-0.3.9/src/elements/animation.rs` (`with_spring`
and retained spring update), and `src/spring.rs` (`SpringConfig`,
`SpringAnimation`, settling).

Unit validation passed as part of 167 passing tests, zero failures across 12
binaries in `target/headstart/milestone56-57-tests.log`. The M56/M57 Headstart
build passed in `target/headstart/milestone56-57-build.log`; all three pinned
dependencies were fresh according to
`target/headstart/milestone56-57-reuse-report.json`.

Bounded native observations are recorded in
`target/headstart/milestone57-spring-gui.json`: the initial target appeared full
opacity; toggling to 0.15 dimmed the child; its button remained clickable and its
activity count advanced; toggling back restored the brighter appearance and
retained count. After enabling reduced motion, a later observation showed the
0.15 target dimmed; motion was then allowed again. The reviewed M56/M57 bundle
SHA-256 is `ede2a1525425256ab1d665c44165ebe50148bf0cf8fc62f7dcc03dc44f03c322`.

Acceptance is qualitative: there are no exact pixel alpha measurements, frame
samples, or latency data. Interruption velocity continuity and eventual scheduler
quiescence were not runtime-measured; reduced-motion immediate timing was not
proved. Custom physics, `.from`, and epsilon configurations were covered by unit
tests but were not all GUI-tested. No CPU/GPU performance improvement is claimed.

## Native animation child identity acceptance sample

`stories/animation_identity_gallery.rs` exercises the animation-owned child
scope repair under native rendering. It contains two sibling SpringOpacity
wrappers and two sibling AnimatedOpacity wrappers, all using the identical
explicit TextField child ID with independent root bindings. Wrapper A/B IDs are
also reused across the two animation families to exercise their typed scopes.
Fixed-width Containers supply layout without adding a retained/keyed component
boundary that could mask the animation wrapper's namespace responsibility.

Edit four distinct values and inspect four separate mirrors. Update the parent
and wait beyond fade completion: values must remain independent. Omit/restore
A wrappers: root-bound A text should remain, while the B fields stay mounted.
External replacement must update only A. No nested owner was inserted merely to
count lifecycle events, because its extra identity scope would weaken this test.
Omission/remount semantics are checked through controls and the bound mirrors;
this sample does not claim measured native InputState mount/release counts.

The namespace fix passed native sibling-gallery acceptance in
`target/headstart/milestone57-animation-identity-gui.json`, bundle SHA-256
`bc184cdda3d2e0f0e3d134f288a3e808153ee4febc74ac2defbe6410826f8dbd`. Two
SpringOpacity and two AnimatedOpacity wrappers intentionally reused one child
TextField ID. Distinct edits stayed separate after parent updates; omitting and
restoring A preserved its bound values and left B values intact; external
replacement changed only A; and Spring B Undo/Redo affected only Spring B.
Undo grouping and exact per-field native entity lifecycle counts were not
measured. This gallery did not measure animation motion, alpha, or velocity.
The earlier bounded M57 opacity observations remain valid only for their
reviewed bundle.

## Spring scheduling probe (M60)

The opt-in probe and verifier are present at
`crates/rustui/examples/support/spring_probe.rs` and
`scripts/verify_spring_probe.py`. Final verification passed in
`target/headstart/spring-opacity-verification-63179-1792336863069682000.json`;
the verifier log is `target/headstart/milestone60-probe-verification-final.log`.
Normal-mode root host lowerings were 3, 13, 15, 26, 42, 42 across the six
samples, showing growth in both passive motion windows and a flat final
quiescence window. Reduced-motion counts were 1, 1, 2, 2, 2, 2, with both
passive windows and final quiescence flat. The initial verifier attempt's float
precision mismatch and saved run artifacts remain in
`target/headstart/milestone60-probe-verification.log` and its referenced JSON
files; the final verifier accepts the emitted f32 values within tolerance.

The probe counts RustUI root host lowerings only. They do not measure GPUI
frames, presented alpha, or velocity continuity. See
`docs/tasks/SPRING_OPACITY_PROBE.md`.
