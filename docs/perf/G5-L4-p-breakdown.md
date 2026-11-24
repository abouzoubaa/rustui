# G5-L4 handler-to-GPU breakdown

## Method

P starts at the native Cocoa handler entry and ends at Metal’s
`GPUEndTime`. The opt-in `RUSTUI_G5_P_BREAKDOWN=1` markers are enabled
alongside the existing buffered latency recorder. Scope markers write to
a fixed buffer and serialize at the same exit flush. The Metal completion
handler records `GPUStartTime` and `GPUEndTime` in the host timebase.

The original five matched sync-off Lab and bare sets are from the L2
matrix. The five sync-off control runs from L3b provide a second RustUI
check with L3 enabled. Those captures lack the new phase markers. The new
five matched Lab/bare pairs use the same diagnostic release build,
alternating Lab-first and bare-first, with 60 seconds after warmup at
10 native Cocoa key events/s (posted events, direct launch). Both arms
verified display sync off in every measured window, with immediate and
draw-first enabled, L3 switches on,
damage off, buffered logging and the passive display link off. The L2
display-sync patch was used only to build the timing binaries.

A hit is a matched version with `0 ≤ SCK displayTime − GPU end < 8.333 ms`.
The threshold uses 120 Hz, the nominal rate supported by the earlier
[L1 passive-link observation](G5-L1-latency-budget.md). The passive-link-off files
contain no independent per-key physical refresh schedule, so this
threshold classifies WindowServer observations. Missing joins and negative
clock margins are excluded from the rate denominator and reported separately.
The join uses the first usable Metal observation and the first decoded SCK
version with the same key tag. It uses neither callback arrival nor submit
time as display time.

The phase calculation assigns nested compute-layout intervals to layout
once and excludes them from prepaint. Scene finalization and the
scene-to-instance copy are separate stages. Unattributed CPU work around
the draw and encoder stays in residual rows. The interval from commit
return to GPU start is a signed queue interval. The per-frame sum is
checked against P before aggregating. Phase percentiles do not add to
the total percentile.

## Screen update classification

P values below are median / p95 in milliseconds. Each dataset has 3,000 keys.

| Capture set | Hits | Misses | Unobserved | Negative margin | Miss rate | P on hits | P on misses |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Original sync-off RustUI (L2) | 1965 | 970 | 65 | 0 | 33.05% | 4.781 / 6.267 | 4.835 / 8.744 |
| Original sync-off bare (L2) | 2418 | 582 | 0 | 0 | 19.40% | 2.226 / 3.228 | 2.022 / 4.369 |
| RustUI L3b control with L3 enabled | 2013 | 945 | 42 | 0 | 31.95% | 4.393 / 5.880 | 4.407 / 9.609 |
| Instrumented RustUI | 2005 | 962 | 33 | 0 | 32.42% | 4.421 / 5.803 | 4.438 / 7.590 |
| Instrumented bare | 2348 | 652 | 0 | 0 | 21.73% | 2.245 / 3.380 | 2.017 / 3.342 |

## Phase differences

The difference is RustUI minus bare, in milliseconds. Rows are sorted by
p95 difference. Only keys with a complete phase trace enter this table.

| Phase | RustUI median / p95 | Bare median / p95 | Difference median / p95 |
| --- | ---: | ---: | ---: |
| Native handler and state update | 1.293 / 2.456 | 0.749 / 1.240 | +0.545 / +1.216 |
| Prepaint excluding nested layout | 0.722 / 0.967 | 0.020 / 0.047 | +0.703 / +0.920 |
| GPU execution | 0.678 / 0.725 | 0.326 / 0.350 | +0.352 / +0.375 |
| Layout request and compute | 0.297 / 0.432 | 0.076 / 0.182 | +0.221 / +0.250 |
| Paint and scene construction | 0.235 / 0.319 | 0.032 / 0.078 | +0.203 / +0.241 |
| Draw end → presentation attempt | 0.109 / 0.382 | 0.072 / 0.313 | +0.037 / +0.069 |
| Other CPU draw setup / teardown | 0.081 / 0.113 | 0.025 / 0.061 | +0.056 / +0.051 |
| Frame finalization and scene sort | 0.039 / 0.049 | 0.003 / 0.007 | +0.036 / +0.041 |
| AppKit dispatch return → draw | 0.136 / 0.290 | 0.135 / 0.279 | +0.000 / +0.011 |
| Scene to Metal instances | 0.006 / 0.010 | 0.003 / 0.007 | +0.004 / +0.003 |
| Drawable acquisition | 0.034 / 0.064 | 0.036 / 0.078 | -0.002 / -0.014 |
| Other CPU presentation work | 0.045 / 0.068 | 0.055 / 0.116 | -0.010 / -0.048 |
| Metal encode | 0.143 / 0.216 | 0.125 / 0.266 | +0.018 / -0.050 |
| Commit return → GPU start | 0.445 / 0.612 | 0.510 / 0.733 | -0.065 / -0.121 |

Complete phase traces: 2,967 RustUI and 3,000 bare. The recorder reported
zero dropped markers in all ten runs.

The three largest p95 differences were:

1. **Native handler and editor state, +1.216 ms.**
   `native_send_event` in `crates/rustui-gpui-fast-macos/src/platform.rs`
   encloses the AppKit dispatch. The editor path calls
   `InputBaseState::replace_text_in_range` in
   `crates/rustui-gpui-base/src/input/base/state.rs`. The separate
   [L3 profile](G5-L3-trim-p.md) identified `TextWrapper::update_wrapped`
   and `InputBaseState::update_fold_candidates_incremental` inside this
   path. L4 measures the combined handler span.
2. **Editor prepaint, +0.920 ms.** `Window::draw_roots` in
   `crates/rustui-gpui-fast/src/window.rs` drives the element prepaint.
   `TextElement::prepaint` and `TextElement::layout_lines` in
   `crates/rustui-gpui-base/src/input/base/element.rs` are the
   editor-specific contributors identified by the L3 profile. L4
   subtracts nested GPUI layout compute from the prepaint span.
3. **Main Metal command buffer execution, +0.375 ms.**
   `MetalRenderer::encode_frame` calls `draw_primitives_to_texture` in
   `crates/rustui-gpui-fast-apple/src/metal_renderer.rs`. L4 measures
   the resulting whole command buffer on the GPU. Individual draw
   batches and shaders were outside the measurement resolution.

In the original RustUI captures, the difference between hit and miss P
medians was 0.054 ms, while miss P p95 was 2.477 ms higher. This measures
the association with missing the threshold; the two cohorts also differ
in their GPU-completion-to-WindowServer interval by definition.

## Host load

Corrected background CPU was checked before each new pair. A pair above
5% is labelled loaded. Each capture also retains its own before/after
load samples.

| Pair | Background before pair | Label |
| --- | ---: | --- |
| 1 | 10.09% | loaded |
| 2 | 10.78% | loaded |
| 3 | 8.55% | loaded |
| 4 | 9.35% | loaded |
| 5 | 21.84% | loaded |

## Reproducing the join

Run the analyzer on the saved capture logs, separately for Lab and bare.
Omit `--detailed` on historical captures. The CSV contains a row per input
version, including unobserved keys.

```sh
python3 scripts/g5_p_breakdown.py --refresh-hz 120 --detailed \
  --csv /path/to/per-key.csv /path/to/capture.log
```

The local result directory is `target/headstart/g5-l4/`.
`existing-lab.csv`, `existing-bare.csv`, and `existing-l3b.csv` contain
the historical classifications. `new-lab.csv` and `new-bare.csv` include
phase durations for the instrumented sets.
