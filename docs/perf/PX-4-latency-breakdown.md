# PX-4 — latency breakdown

Measurement only; no scheduling, rendering, frame-latency or drawable-count setting is changed. Base `p-integration`: 78d896eb09769c2618eeb6f25fbd08e999517497. Pinned Rust 1.98.0, release build, damage off, buffered logging. `RUSTUI_LATENCY_BREAKDOWN=1` opts into new records; the G0 state tags additionally use `RUSTUI_G0_LATENCY=1`.

## Method

All pipeline events carry raw `mach_absolute_time` ticks; the timebase is recorded once. The handled timestamp is taken before scripted dispatch. Invalidation hooks record notify/refresh demand. CAMetalDisplayLink arrival is captured before the window lock. Each CPU draw and each presentation attempt has its own ID. A drawable acquisition gets a unique ID even if its pointer is later reused. Command commit return and addCompletedHandler arrival are recorded separately. Actual MTLDrawable.presentedTime is calibrated onto mach ticks using callback host time; no submit/completion timestamp stands in for display.

For the first valid displayed drawable containing a painted input version, stage 1 uses the first subsequent invalidation, stage 2 the next native callback, and stage 3 the interval from that callback to the draw used by the presented attempt. Thus stage 3 includes later deferred callbacks if any. Stage 4 is elapsed draw-to-submit time and can include a gap before a later presentation attempt; it is not pure CPU encoding time. Draws outside a callback or without a qualifying request are reported with incomplete chains; callback ID zero cannot borrow an unrelated earlier callback. Stages 2–3 are unavailable for those draws, while draw-to-submit/GPU/display intervals remain reported. Completion-handler arrival can follow actual display: signed stage 6 values are preserved and counted, never clamped. GPU end-to-display from Metal GPU timestamps is a separate diagnostic. Missing/coalesced states and unavailable presentedTime values remain explicit.

Acquisition counts include acquired drawables until their presented handler arrives; unused drawables are retired when the platform releases them. An unavailable presentation callback retires the acquisition but is excluded from screen-delay statistics. This is app-visible lifetime accounting; it is not a private WindowServer queue measurement. Settings changes carry mach timestamps, so startup/live-resize state cannot replace the measured-window state. Per-window settings read maximumDrawableCount, displaySyncEnabled, the layer presentsWithTransaction property and renderer synchronous flag separately, actual frame source, preferredFrameLatency, preferredFrameRateRange, and the screen's reported maximum refresh rate. In-flight counts include the just-acquired drawable; cadence and acquisition histograms exclude warmup using mach timestamps in the measurement boundary markers. Callback intervals independently report delivered cadence.


## Controlled measurements

2026-11-21, MacBookPro18,2 / M1 Max / 64 GiB, macOS 27.0 (26A5425a). One pinned 1.98.0 release Lab build, fixed 1200 × 800 points at scale 2. Source code was frozen at `b2857fb49920cab2d4b66bf1f86a82df1ad7fad9`; later changes to this ticket document only the results. The reference-only root-borrow setup failure was corrected before measurement; its partial capture is excluded. AppKit and bare references each completed one descriptive 60-second run after normal warmup. Their different workloads/endpoints are not used for a Stage P CPU verdict.

The four Lab scenes each ran five matched off/on pairs, ordered off→on, on→off, off→on, on→off, off→on; each measured 60 seconds after normal warmup. Both arms used `RUSTUI_DAMAGE=0 RUSTUI_FRAME_LOG=buffered RUSTUI_FRAME_PROFILE=1 RUSTUI_G0_LATENCY=1`; only `RUSTUI_LATENCY_BREAKDOWN=0/1` changed. No local compilation, profiling or other test workload overlapped timing. The unrelated indexer was paused; ordinary macOS services remained active. Raw captures remain in ignored local artifacts, not Git.

### Observer CPU verdict

Deciding metric: measured-window getrusage user + system seconds per attempted present. These verdicts measure diagnostic overhead, not a rendering optimization. [Owner rule](MEASUREMENT_RULES.md); unrounded values determine the verdict.

| Scene | Off median ms/attempt | On median ms/attempt | Five paired changes % | Median % | Min % | Max % | Verdict |
|---|---:|---:|---|---:|---:|---:|---|
| typing | 13.791191 | 13.777268 | -1.734, -0.971, +6.927, -0.081, +9.463 | -0.081 | -1.734 | +9.463 | **INCONCLUSIVE** |
| hover | 2.577325 | 2.598064 | +2.745, +1.898, -2.812, +0.096, +3.073 | +1.898 | -2.812 | +3.073 | **INCONCLUSIVE** |
| drag | 1.486376 | 1.487455 | -1.545, +0.461, -0.507, -1.169, +5.938 | -0.507 | -1.545 | +5.938 | **INCONCLUSIVE** |
| log | 3.862927 | 3.940186 | -1.264, +2.712, +1.821, +3.365, +0.858 | +1.821 | -1.264 | +3.365 | **INCONCLUSIVE** |

### Six-stage timing (diagnostic on)

Median and p95 are pooled from individual matched input samples across the five on-captures, not averaged percentiles. The sample count accompanies each stage. Medians/percentiles are not additive. Stages 2–3 apply only to draws with a real native callback scope; synchronous/outside-callback draws are counted below and do not borrow a nearby callback. Stage 6 is signed and uses actual displayed drawable time; a negative sample means the completion callback arrived after display.

| Scene | Stage | Samples | Median ms | p95 ms |
|---|---|---:|---:|---:|
| typing | 1 handling → request | 2594 | 0.117 | 0.740 |
| typing | 2 request → next callback | 2594 | 11.270 | 15.997 |
| typing | 3 callback → draw | 2594 | 0.008 | 0.015 |
| typing | 4 draw → commit return | 2594 | 0.775 | 1.425 |
| typing | 5 commit → completion callback | 2594 | 1.328 | 3.663 |
| typing | 6 completion callback → actual display | 2594 | 43.427 | 54.208 |
| hover | 1 handling → request | 17992 | 0.011 | 0.022 |
| hover | 2 request → next callback | 17992 | 2.712 | 5.298 |
| hover | 3 callback → draw | 17992 | 0.010 | 0.017 |
| hover | 4 draw → commit return | 17992 | 2.856 | 7.649 |
| hover | 5 commit → completion callback | 17992 | 2.339 | 4.164 |
| hover | 6 completion callback → actual display | 17992 | 37.830 | 38.757 |
| drag | 1 handling → request | 17970 | 0.012 | 0.022 |
| drag | 2 request → next callback | 17970 | 2.493 | 6.514 |
| drag | 3 callback → draw | 17970 | 0.012 | 0.020 |
| drag | 4 draw → commit return | 17970 | 0.638 | 7.763 |
| drag | 5 commit → completion callback | 17970 | 3.494 | 4.492 |
| drag | 6 completion callback → actual display | 17970 | 37.704 | 38.939 |
| log | 1 handling → request | 3000 | 0.011 | 0.017 |
| log | 2 request → next callback | 3000 | 4.576 | 8.293 |
| log | 3 callback → draw | 3000 | 0.018 | 0.025 |
| log | 4 draw → commit return | 3000 | 2.333 | 3.334 |
| log | 5 commit → completion callback | 3000 | 1.296 | 3.216 |
| log | 6 completion callback → actual display | 3000 | 38.675 | 47.911 |
| bare Fast | 1 handling → request | 575 | 0.021 | 0.028 |
| bare Fast | 2 request → next callback | 575 | 4.320 | 7.975 |
| bare Fast | 3 callback → draw | 575 | 0.017 | 0.025 |
| bare Fast | 4 draw → commit return | 575 | 0.384 | 0.668 |
| bare Fast | 5 commit → completion callback | 575 | 1.264 | 3.782 |
| bare Fast | 6 completion callback → actual display | 575 | 47.580 | 48.894 |

### Display endpoints and coverage

| Scene / method | Handled | Matched display samples | Complete native chains | Outside-callback draws | No valid displayed state | Actual display median / p95 ms | GPU end → display median / p95 ms |
|---|---:|---:|---:|---:|---:|---|---|
| typing / Metal presentedTime | 3000 | 2594 | 2594 | 0 | 406 | 57.687 / 68.999 | 43.686 / 54.426 |
| hover / Metal presentedTime | 36000 | 17992 | 17992 | 0 | 18008 | 46.358 / 51.936 | 38.164 / 39.063 |
| drag / Metal presentedTime | 36000 | 17970 | 17970 | 0 | 18030 | 47.164 / 51.671 | 38.119 / 39.967 |
| log / Metal presentedTime | 3000 | 3000 | 3000 | 0 | 0 | 49.459 / 58.143 | 38.965 / 48.146 |
| bare Fast / Metal presentedTime | 576 | 575 | 575 | 0 | 1 | 51.941 / 57.811 | 48.110 / 49.131 |

**AppKit reference:** 599/599 handled versions matched the first CADisplayLink callback after NSTextView.draw. The handling→first-post-draw display-link callback proxy is **16.592 ms median / 20.688 ms p95**; handling→draw was 8.488/8.905 ms, draw→callback 7.805/12.135 ms. The input also changed the marker layer’s contents. This is **not observed scanout**: exact AppKit key-to-screen delay is **not measured**. Do not subtract this proxy from Metal presentedTime to claim framework overhead. The bare reference is an ASCII-only input at its observed cadence; the Lab typing workload is a 10,000-line editor. Their difference is not an isolated RustUI-wrapper cost.

Painted-but-unpresented and overwritten/coalesced versions remain explicit in the per-capture table below. Zero presentedTime callbacks are not assigned a delay. The acquired-drawable observer also receives zero callbacks for unused released drawables, so its full-lifecycle zero-callback total is not a count of lost input frames. GPU-end→display is separate from completion-handler arrival, allowing callback lateness to be distinguished from post-GPU waiting.

### Per-window configuration during the measurement

Each launch has one window and one measured Metal surface; WindowId(1v1) is process-local. The table pools only runs with identical settings; startup transaction-present changes are preserved in the raw timeline and excluded from the steady-state values. Reported maximum Hz is the screen maximum; callback cadence is measured separately and can differ.

| Scene | max drawables | display sync | layer transaction presents | renderer synchronous | Metal display link | preferred latency | rate range min/max/preferred | screen max Hz | callback interval median ms | in-flight acquisition histogram |
|---|---:|---:|---:|---:|---|---:|---|---:|---:|---|
| typing | 3 | 1 | 0 | 0 | yes | 2 | 0/0/0 | 120 | 8.360 | 1: 2604, 2: 5178 |
| hover | 3 | 1 | 0 | 0 | yes | 2 | 0/0/0 | 120 | 8.334 | 1: 25, 2: 6, 3: 7, 4: 37, 5: 17974, 6: 17924, 7: 19 |
| drag | 3 | 1 | 0 | 0 | yes | 2 | 0/0/0 | 120 | 8.336 | 1: 24, 2: 11, 3: 31, 4: 116, 5: 17875, 6: 17755, 7: 68 |
| log | 3 | 1 | 0 | 0 | yes | 2 | 0/0/0 | 120 | 9.836 | 1: 2880, 2: 2872 |
| bare Fast | 3 | 1 | 0 | 0 | yes | 2 | 0/0/0 | 120 | 9.129 | 1: 577, 2: 575 |

The in-flight count includes the newly acquired drawable and retires unused releases; it measures app-visible acquisitions awaiting presentation/release notification, not private compositor buffering. Continuous hover/drag show 5–7 pending acquisition IDs despite maximumDrawableCount=3. This is an observed discrepancy, not validated physical texture-pool occupancy. It must not be used alone to infer the internal queue size; a later pool experiment should also identify the underlying textures and distinguish dropped outcomes. Callback interval above is the median of the five per-run medians (one run for bare), within the measured windows; idle/demand gaps remain part of that cadence.

### Per-capture CPU and screen-delay diagnostics

CPU seconds exclude warmup and exit-buffer flushing. Slow frames here are draw+present elapsed CPU >8.3 ms, not presentation gaps or input-delay counts. Delay statistics use valid displayed versions only.

| Scene | Pair | Arm | User s | Sys s | CPU s | Attempts | CPU s/attempt | Draw+present ms/attempt | CPU frames >8.3 ms | Shown/inputs | Delay median/p95/max ms | Coalesced / painted without presentation |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|
| typing | 1 | off | 6.548657 | 0.530623 | 7.079280 | 521 | 0.013587869 | 0.834175 | 0 | 521/600 | 57.729/69.894/70.754 | 0/79 |
| typing | 1 | on | 6.449596 | 0.533646 | 6.983242 | 523 | 0.013352279 | 0.895817 | 0 | 523/600 | 57.668/67.232/72.337 | 0/77 |
| typing | 2 | off | 6.626537 | 0.543126 | 7.169663 | 519 | 0.013814380 | 0.865337 | 0 | 519/600 | 57.721/67.710/74.766 | 0/81 |
| typing | 2 | on | 6.603498 | 0.523941 | 7.127439 | 521 | 0.013680305 | 0.831766 | 0 | 521/600 | 57.562/67.706/71.802 | 0/79 |
| typing | 3 | off | 6.686796 | 0.525211 | 7.212007 | 518 | 0.013922793 | 0.840183 | 0 | 518/600 | 57.727/68.530/70.779 | 0/82 |
| typing | 3 | on | 7.141923 | 0.495248 | 7.637171 | 513 | 0.014887273 | 0.835967 | 0 | 513/600 | 58.192/68.996/74.455 | 0/87 |
| typing | 4 | off | 6.599296 | 0.544541 | 7.143837 | 518 | 0.013791191 | 0.864829 | 0 | 518/600 | 57.718/69.456/74.762 | 0/82 |
| typing | 4 | on | 6.596121 | 0.541896 | 7.138017 | 518 | 0.013779956 | 0.870516 | 0 | 518/600 | 57.668/69.200/70.732 | 0/82 |
| typing | 5 | off | 6.118078 | 0.477106 | 6.595184 | 524 | 0.012586229 | 0.822167 | 0 | 524/600 | 55.844/66.971/70.732 | 0/76 |
| typing | 5 | on | 6.606118 | 0.544284 | 7.150402 | 519 | 0.013777268 | 0.839556 | 0 | 519/600 | 56.609/70.071/70.749 | 0/81 |
| hover | 1 | off | 14.046473 | 4.578627 | 18.625100 | 7197 | 0.002587898 | 1.723933 | 0 | 3599/7200 | 46.317/51.757/52.098 | 1251/2350 |
| hover | 1 | on | 14.469674 | 4.669413 | 19.139087 | 7198 | 0.002658945 | 1.775014 | 0 | 3598/7200 | 46.375/51.887/54.091 | 1309/2293 |
| hover | 2 | off | 13.785164 | 4.470529 | 18.255693 | 7160 | 0.002549678 | 1.682281 | 1 | 3587/7200 | 46.549/51.984/73.565 | 1550/2063 |
| hover | 2 | on | 14.128580 | 4.564487 | 18.693067 | 7195 | 0.002598064 | 1.710639 | 1 | 3599/7200 | 46.505/51.963/57.609 | 1533/2068 |
| hover | 3 | off | 14.090000 | 4.500669 | 18.590669 | 7199 | 0.002582396 | 1.712475 | 0 | 3598/7200 | 46.414/51.954/55.579 | 1432/2170 |
| hover | 3 | on | 13.695592 | 4.372382 | 18.067974 | 7199 | 0.002509789 | 1.661095 | 0 | 3598/7200 | 46.360/51.968/52.228 | 1416/2186 |
| hover | 4 | off | 13.819204 | 4.507097 | 18.326301 | 7197 | 0.002546381 | 1.680578 | 0 | 3598/7200 | 46.331/51.935/52.468 | 1428/2174 |
| hover | 4 | on | 13.858013 | 4.488470 | 18.346483 | 7198 | 0.002548831 | 1.693994 | 0 | 3599/7200 | 46.287/51.966/54.071 | 1394/2207 |
| hover | 5 | off | 13.972267 | 4.579315 | 18.551582 | 7198 | 0.002577325 | 1.711260 | 1 | 3602/7200 | 46.160/51.848/56.587 | 1301/2297 |
| hover | 5 | on | 14.434786 | 4.684187 | 19.118973 | 7197 | 0.002656520 | 1.772234 | 1 | 3598/7200 | 46.310/51.721/54.129 | 1329/2273 |
| drag | 1 | off | 5.388728 | 5.445666 | 10.834394 | 7199 | 0.001504986 | 0.429503 | 0 | 3602/7200 | 47.195/51.886/54.114 | 1160/2438 |
| drag | 1 | on | 5.261670 | 5.331217 | 10.592887 | 7149 | 0.001481730 | 0.410477 | 0 | 3588/7200 | 47.197/51.933/104.492 | 1378/2234 |
| drag | 2 | off | 5.317007 | 5.380443 | 10.697450 | 7197 | 0.001486376 | 0.424594 | 0 | 3599/7200 | 47.110/51.299/54.777 | 1283/2318 |
| drag | 2 | on | 5.355646 | 5.394124 | 10.749770 | 7199 | 0.001493231 | 0.418510 | 0 | 3600/7200 | 47.053/51.236/57.548 | 1260/2340 |
| drag | 3 | off | 5.344177 | 5.417098 | 10.761275 | 7198 | 0.001495037 | 0.420723 | 0 | 3599/7200 | 46.796/51.119/54.102 | 1335/2266 |
| drag | 3 | on | 5.369646 | 5.340028 | 10.709674 | 7200 | 0.001487455 | 0.421370 | 0 | 3600/7200 | 46.962/51.277/55.504 | 1365/2235 |
| drag | 4 | off | 5.222322 | 5.317047 | 10.539369 | 7198 | 0.001464208 | 0.409164 | 0 | 3598/7200 | 47.250/51.811/61.151 | 1456/2146 |
| drag | 4 | on | 5.273467 | 5.142662 | 10.416129 | 7198 | 0.001447087 | 0.413051 | 0 | 3603/7200 | 47.230/51.555/55.575 | 1457/2140 |
| drag | 5 | off | 5.128197 | 5.149826 | 10.278023 | 7198 | 0.001427900 | 0.397517 | 0 | 3599/7200 | 47.237/51.915/55.444 | 1421/2180 |
| drag | 5 | on | 5.387911 | 5.397566 | 10.785477 | 7130 | 0.001512690 | 0.427748 | 0 | 3579/7200 | 47.265/51.900/108.594 | 1483/2138 |
| log | 1 | off | 1.627641 | 0.666755 | 2.294396 | 576 | 0.003983326 | 2.326755 | 0 | 600/600 | 49.821/58.184/149.270 | 0/0 |
| log | 1 | on | 1.603823 | 0.657638 | 2.261461 | 575 | 0.003932976 | 2.252758 | 0 | 600/600 | 49.417/58.174/162.372 | 0/0 |
| log | 2 | off | 1.569386 | 0.642501 | 2.211887 | 575 | 0.003846760 | 2.227554 | 0 | 600/600 | 49.415/57.816/158.091 | 0/0 |
| log | 2 | on | 1.615447 | 0.656433 | 2.271880 | 575 | 0.003951096 | 2.288241 | 0 | 600/600 | 49.437/58.143/149.408 | 0/0 |
| log | 3 | off | 1.571333 | 0.640487 | 2.211820 | 575 | 0.003846643 | 2.207968 | 0 | 600/600 | 49.423/57.813/158.146 | 0/0 |
| log | 3 | on | 1.603584 | 0.648513 | 2.252097 | 575 | 0.003916690 | 2.256226 | 0 | 600/600 | 52.503/57.816/153.346 | 0/0 |
| log | 4 | off | 1.577634 | 0.647412 | 2.225046 | 576 | 0.003862927 | 2.193480 | 0 | 600/600 | 49.430/58.156/151.712 | 0/0 |
| log | 4 | on | 1.633580 | 0.662355 | 2.295935 | 575 | 0.003992930 | 2.319399 | 0 | 600/600 | 49.415/58.166/153.680 | 0/0 |
| log | 5 | off | 1.608722 | 0.637616 | 2.246338 | 575 | 0.003906675 | 2.264377 | 0 | 600/600 | 49.264/57.501/154.035 | 0/0 |
| log | 5 | on | 1.632809 | 0.636738 | 2.269547 | 576 | 0.003940186 | 2.284640 | 0 | 600/600 | 49.464/57.713/157.538 | 0/0 |

### Ranked waits and the smallest next tests

1. **Post-GPU display wait.** The six-stage table and independent bare-Fast reference locate this wait after GPU completion, rather than in view building. This identifies the pipeline segment, not the exact cause inside CAMetalDisplayLink/Core Animation/WindowServer. First test: one opt-in `setPreferredFrameLatency(1.0)` call at MetalFrameSource construction, preserving the default when off. Apple permits only 1 or 2 frames and says windowed macOS can add latency beyond that request. Measure stage 6, total input-to-screen median/p95, process CPU per attempt and slow-frame counts with the same five-pair method; no saving is promised. [Apple preferredFrameLatency](https://developer.apple.com/documentation/quartzcore/cametaldisplaylink/preferredframelatency).

2. **Drawable-pool contribution to that same segment.** Separately test `maximumDrawableCount=2` at layer configuration; do not combine it with the first knob in the first comparison. The current pool maximum and app-visible in-flight counts are in the table; they do not prove how many frames the compositor retains. Apple supports 2 or 3 drawables. Measure acquisition stalls/cadence alongside stage 6 and input-to-screen delay. [Apple maximumDrawableCount](https://developer.apple.com/documentation/quartzcore/cametallayer/maximumdrawablecount).

3. **Request→callback wait.** This interval includes the remainder of the foreground input turn as well as delivery/wakeup delay, so it is not all display-link sleep. PX-3’s already-approved highlighter experiment comes next. A later input-tick/frame-request experiment should log handler return and targetPresentationTimestamp before attributing the remaining wait or changing scheduling; preserve the outside-callback distinction. No request-timing or input-tick behavior is changed here.

4. **Draw/encoding and GPU work.** These measured segments are smaller than the post-GPU tail on the reference. Optimize them only with scene evidence; partial culling and G2 work are not part of this ticket. Normal measured transaction-present flags are read, not inferred from startup or changed.

Each future fix needs its own candidate patch/branch/PR and matched measurements. The AppKit proxy does not establish an exact observed-screen floor, and P4 does not measure private WindowServer queues or CAMetalDisplayLink targetPresentationTimestamp. None of these hypotheses is implemented in PX-4.

### Validation

`scripts/check.sh`: 1,874 total tests (1,867 passed, 7 ignored) across all seven groups. `./scripts/verify-release.sh source`: 684 passed. Focused real-mach lifetime/scope/correlation tests and both reference builds passed. Review findings about callback scope and acquisition timing were fixed before these captures. Final source CI must be green before merge. All switches remain off by default; no runtime pacing policy changes, no G2 telemetry completion, no G2-8b, PX-3 or P1 work is included here.
