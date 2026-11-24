# G5-L3 trim handler-to-GPU time

G5-L1 merged into `g5-integration` before this ticket. The L3 benchmark
uses direct launch with the passive display link off, damage off, buffered
logging, and native Cocoa test input at 10 keys/s. Physical typing remains
reserved for the owner session.

The target is handler → GPU end median ≤3.0 ms and p95 ≤4.5 ms. The
ScreenCaptureKit targets are median ≤11 ms and p95 ≤15 ms, with no
process-CPU regression across five alternating matched pairs.

## Candidate

`GPUI_FAST_LATE_DRAWABLE=1` stages scroll and nested tile rasterization and
writes the frame's instance buffers before requesting a drawable in the
immediate or detached whole-frame path with damage off. The renderer still
checks drawable capacity at acquisition and uses the actual drawable size
for the main render pass. The staged path requires initialized intermediate
textures. It leaves dynamic uploads in the final command buffer, where
acquisition failure cannot discard them. The existing headless frame proof
compares staged and ordinary pixels for the same nonempty scene.

The GPUI Kit candidate `RUSTUI_G5_TRIM_EDITOR=1` seeks into the edited
byte range before visiting fold siblings. It uses the same fold overlap,
minimum span, order, and deduplication rules as the reference scan. The
same switch retains bounded exact plain-text wrap measurements across
edits and reuses the unwrapped longest-line measurement when the line and
its font parameters match. Font, tab, and other full layout updates
continue to clear the relevant wrap measurements. Wrapping and fold
projection still update before the first visible frame.

## Native profile

A sample-only release executable links the benchmark helpers so Samply
can use its own loader injection. It uses the same Rust sources, toolchain
and optimization settings as G5-L1. Direct launch, passive link off,
damage off, buffered logging, and the G5-F2/PX-3 switches on. Samply sampled
the main thread at 2 kHz for the 30-second measured typing window after
warmup. ScreenCaptureKit ran in the separate benchmark observer.
Background CPU was 5.81% at the start and 5.26% afterward.

The capture delivered and applied all 300 native Cocoa test events, with
six earlier versions superseded and 294 matched Metal and ScreenCaptureKit
versions. The instrumented capture's median / p95 elapsed times were
2.22 / 5.78 ms before draw, 1.20 / 1.62 ms inside draw, and 5.04 / 8.73 ms
handler → GPU end. The profiled capture is separate from the paired timing
test.

| Stage | RustUI-owned | GPUI Kit | GPUI Fast | AppKit / HIToolbox | Other |
| --- | ---: | ---: | ---: | ---: | ---: |
| Handler entry → draw start | 0.018 | 0.083 | 0.205 | 0.102 | 1.297 |
| CPU draw start → end | 0.001 | 0.074 | 0.305 | 0.018 | 0.650 |

Values are sampled main-thread self CPU ms per matched edit. Other includes
native system libraries, the allocator, dependencies such as Tree-sitter
and Ropey, test code and unresolved frames. The local
`target/headstart/g5-l3/native-profile-functions.csv` lists each sampled
symbol/source-line combination, by stage and group, with self and inclusive
times. The analysis clips the profiler intervals to the mach timestamps of
each matched edit. Inclusive values overlap.

| Stage | Function | Inclusive CPU ms/edit |
| --- | --- | ---: |
| Before draw | GPUI Fast `Window::dispatch_event` | 0.706 |
| Before draw | GPUI Kit `TextWrapper::update_wrapped` | 0.259 |
| Before draw | GPUI Kit `InputBaseState::update_fold_candidates_incremental` | 0.237 |
| Before draw | GPUI Kit `extract_fold_ranges_in_range` | 0.221 |
| Inside draw | GPUI Kit editor `TextElement::prepaint` | 0.276 |
| Inside draw | GPUI Fast `paint_entity_view` | 0.149 |
| Inside draw | GPUI Fast `paint_line` | 0.102 |
| Inside draw | GPUI Kit `TextElement::layout_lines` | 0.097 |

The synchronous fold scan visited siblings beyond the edited byte range.
The measured wrap function already splices unchanged rows. Its width
measurement cache was still cleared on every edit, and the longest-line
lookup missed on every document revision. The candidate changes those
lookups. Validation that can reject an edit still runs before it becomes
visible. The profile did not identify a comparable RustUI-owned listener
cost in the two intervals.

## Matched timing

Each capture used one frozen release build, direct launch, damage off,
passive link off and buffered logging. The G5-F2 and PX-3 switches were
on. Five pairs used 60-second windows after warmup, alternating the
order. Both L3 switches were off for the control and on for the
candidate. All ten pair captures were measured on a loaded machine.

| Arm | Handler → GPU end | Handler → SCK | Handler → Metal | Matched / delivered |
| --- | ---: | ---: | ---: | ---: |
| Control | 4.97 / 7.80 | 13.91 / 20.67 | 24.22 / 31.94 | 2,937 / 3,000 |
| Editor + late drawable | 4.60 / 7.65 | 13.42 / 19.81 | 24.14 / 30.58 | 2,931 / 3,000 |

Latency cells are pooled median / p95 in ms. The paired CPU changes
per attempted present were −3.06%, −4.70%, −4.46%, −1.19%, and −3.58%,
an INCONCLUSIVE P verdict. Total CPU/s changes were −2.88%, −5.52%,
−5.27%, +0.01%, and −4.18%, also INCONCLUSIVE. The text and order
oracles passed with zero lost edits and zero order violations. No
measured-window drawable acquisition exceeded 2 ms.

| Pair | Control P | Candidate P | Control SCK | Candidate SCK | Control Metal | Candidate Metal | Background CPU start→after, control / candidate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 1 | 4.98 / 7.48 | 4.54 / 7.85 | 13.90 / 20.57 | 12.74 / 19.73 | 24.18 / 31.58 | 24.09 / 32.08 | 15.95→17.82% / 12.42→15.79% |
| 2 | 5.01 / 7.80 | 4.56 / 7.34 | 14.21 / 21.17 | 13.39 / 19.88 | 25.29 / 32.22 | 24.14 / 30.53 | 12.24→15.42% / 11.78→17.83% |
| 3 | 5.03 / 8.20 | 4.70 / 8.37 | 14.21 / 20.75 | 13.46 / 21.17 | 26.13 / 31.70 | 24.14 / 32.27 | 10.98→16.94% / 13.60→16.16% |
| 4 | 4.89 / 7.56 | 4.69 / 7.88 | 13.42 / 20.31 | 13.81 / 19.66 | 24.20 / 31.55 | 24.19 / 30.90 | 11.14→15.74% / 13.69→16.40% |
| 5 | 4.96 / 7.60 | 4.54 / 6.11 | 13.63 / 20.19 | 13.82 / 17.88 | 24.19 / 31.38 | 24.24 / 29.02 | 7.90→25.64% / 20.90→18.07% |

The separate 30-second diagnostics show the sequence of changes. These
captures were measured on a loaded machine, and their order was fixed.

| Step | P | Handler → SCK | Handler → Metal | Background CPU start→after |
| --- | ---: | ---: | ---: | ---: |
| Control | 5.04 / 7.40 | 13.67 / 20.82 | 24.15 / 31.78 | 5.46→11.47% |
| Editor reuse only | 4.52 / 7.58 | 12.78 / 19.66 | 24.12 / 29.82 | 4.92→17.28% |
| Editor reuse + late drawable | 4.45 / 7.98 | 12.28 / 19.59 | 24.03 / 32.23 | 17.51→23.31% |

The candidate lowered the median P and the SCK p95, but missed both
absolute targets. In the first control capture, the outer Cocoa handler
itself took 2.03 / 4.88 ms. The time after handler return to draw start
was 0.16 / 0.27 ms. The profile still attributes some CPU to native
keyboard routing and a pre-dispatch GPUI draw. Moving the post-event
wake earlier would not remove that in-handler work. The candidate
remains draft. The next L2 experiment uses the merged L1 baseline.
