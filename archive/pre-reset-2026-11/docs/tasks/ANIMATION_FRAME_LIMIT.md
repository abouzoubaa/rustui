# Optional native animation scheduling limit

`AnimatedOpacity::max_fps(NonZeroU16)` forwards an optional scheduling limit to
pinned GPUI's `Animation::with_max_fps`. The default remains `None`, so existing
opacity animations continue requesting every native animation frame.

```rust
use std::{num::NonZeroU16, time::Duration};
use rustui::prelude::*;

let fade = AnimatedOpacity::new(
    "status-fade", Text("Ready"), 0.0, 1.0, Duration::from_secs(1),
)?.max_fps(NonZeroU16::new(30).expect("30 is nonzero"));
```

Configure the limit before mounting. A later builder call replaces the earlier
limit. The nonzero integer type accepts 1 through 65535; each value converts
exactly to the native float and yields a finite, nonzero scheduling interval.
This API adds no additional validation error or public type export. Endpoints,
duration, one-shot identity semantics, and GPUI reduced-motion behavior remain
the existing animation contract.

## Scheduling and ownership

Pinned source `vendor/gpui-pre-0.3.9/src/elements/animation.rs:65-72` provides the
native option. In `request_layout`, GPUI keeps a pending-notification flag and
schedules its next view notification with its own executor/timer for a finite,
positive configured limit (`animation.rs:451-468`). An unrestricted animation
uses `window.request_animation_frame()` instead. GPUI owns the animation clock,
interpolation, delayed work, and view notification. RustUI only constructs the
native configuration; there is no new scheduler, retained runtime, dependency,
feature change, or upstream edit.

The app-level reduced-motion branch selects the one-shot endpoint and marks
the animation done before the scheduling branch (`animation.rs:407-414`). The
new builder uses that existing native path. This does not establish OS motion
preference propagation or change already-pending native notifications.

This limit controls the animation's own requested updates. Unrelated owner
invalidations can still render it more often, and scheduling delays can make
it update less often. It does not guarantee a hard render-rate cap, displayed
FPS, GPU work reduction, or presentation timing.

## Executed evidence

Source: `crates/rustui/src/modifiers/opacity.rs`.
Two API/model checks inspect the exact native configuration for the unrestricted
default and limits 1, 30, and 65535, including replacement behavior, unchanged
duration/endpoints/one-shot mode, and representable intervals. Both checks
passed within the 135-test suite (0 failures) recorded in
`target/headstart/milestone46-50-tests.log`. The probe build completed in
`target/headstart/milestone50-probe-build.log`.

The executed four-process verifier completed successfully:
`target/headstart/opacity-probe-verification-90647-1792326980117436000.json`.
All run validation-error lists are empty and all runs quiesced.

| Process mode | Baseline | Midpoint | Post-duration | Quiescence |
| --- | ---: | ---: | ---: | ---: |
| `normal` | 3 | 23 | 43 | 43 |
| `reduce_motion` | 1 | 1 | 1 | 1 |
| `limited` (configured 10) | 1 | 5 | 11 | 11 |
| `limited_reduce_motion` (configured 10) | 1 | 1 | 1 | 1 |

The observed post-duration-minus-baseline lowering deltas were 40 unrestricted
and 10 limited; the comparison status is `complete`. Both reduced-motion
processes retained one lowering throughout their sampling windows. These are
isolated RustUI host-lowering observations. They establish no displayed FPS,
GPU or CPU performance change, presentation timing, or captured pixel alpha.

The verifier retained these SHA-256 provenance values:

- Probe executable: `ac0f59d65fdb5f206a20f8422ab8c3f126b4e370610a290b76a8d9874f0b1f3e`
- Probe source: `8043022eb8380272aab8949b8a2302037528e1f257e128a9c71f680e78785312`
- Verifier source: `51ffc5ef5f7ab1d134368644f6fd8d24b7c6de1b3455d246f86b60cd2cecc4c9`

## Opt-in runtime comparison

The existing opacity probe now accepts `--limit-fps`, requesting native
10-FPS scheduling. Combine it with `--reduce-motion` for a fresh-process
reduced-motion check. The fixed 10-FPS selection keeps the existing one-second
animation and midpoint windows useful; it is not a restriction on the public
builder's supported values. Both modes keep the existing bounded static
startup warmup, first-lowering baseline, progression, and quiescence samples.
Sampling only reads diagnostics; it adds no view notifications or frame requests.

Build the example, then run:

```sh
./scripts/cargo-headstart build -p rustui --example opacity_benchmark
python3 scripts/verify_opacity_probe.py --with-frame-limit
```

The runner starts four separate processes: `normal`, `reduce_motion`, `limited`,
and `limited_reduce_motion`. Without `--with-frame-limit`, it preserves the
original two-mode workflow. An alternate already-built executable can be
passed with `--binary /absolute/path/to/opacity_benchmark`.

Each limited result records `configured_max_fps: 10`. The animated run must
show host-lowering progress and post-duration quiescence; the reduced-motion
run must retain the original stable-lowering endpoint contract. The verifier
also compares the limited and unrestricted post-duration-minus-baseline counts.
It accepts the comparison only when the limited run has a positive, smaller
observed lowering delta. Otherwise comparison status stays `unconfirmed` and
verification fails. This is an observation in isolated probe processes, not a
strict wall-clock cap; unrelated owner invalidations can contribute lowerings.
The result supplies neither displayed FPS nor captured pixel opacity.

Python syntax checks passed. The updated validator accepted the historical
normal/reduced results in
`target/headstart/opacity-probe-verification-31323-1792319071087036000.json`.
Synthetic limited/reduced records checked the validator's acceptance and its
rejection of missing configuration or nonquiescent counts. Those model checks
are separate from the executed four-process runtime evidence above.

Frame callback intervals cannot establish rendered or presented frames: the
pinned callback loop runs before conditional drawing, as clarified in
`docs/tasks/P2C_GPUI_TIMING_HOOKS.md`.
