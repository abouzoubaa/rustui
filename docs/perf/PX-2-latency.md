# PX-2 — handled state to displayed drawable

`RUSTUI_G0_LATENCY=1` opts into G0 input/append timing; it is off by default. Use `RUSTUI_G0_LATENCY=1 RUSTUI_DAMAGE=0 ./scripts/run_g0_bench.sh`. The script prints median/p95/max input-to-screen delay, shown/coalesced/unpresented counts, process CPU per attempt, and draw+present frames over 8.3 ms. Input-delay counts over 8.3 ms are separate from slow CPU-phase frames.

The Lab timestamps each scripted input at handling, then publishes its state version only after the synchronous handler completes. Pre-dispatch hit-test paints cannot claim the new state. The core records that version after root painting and carries the painted tag through the present scope. Metal adds a drawable presented handler and reads `MTLDrawable.presentedTime`; command-buffer submission or completion is never used as the screen endpoint. The callback's Core Animation host time measures callback lateness; subtracting that duration maps the actual display instant onto the input recorder's monotonic timeline. Zero/unavailable drawable timestamps are reported and excluded. The exact host-clock SDK symbol is used because CABase is not exposed by the pinned binding features; QuartzCore was already linked. All owned source changes are regenerated from the reversible candidate patch.

Analysis sorts actual display instants, including out-of-order callbacks, and selects the first displayed drawable covering each input's painted version. Overwritten discrete versions are reported as coalesced/unpainted, never assigned another state's display time. The fixed-size log scene conservatively tags its newest 20 visible rows, so multiple appends can be shown by one drawable. The window is 1200 × 800 at scale 2; the 24-point log rows have a viewport taller than 20 rows. A painted version with no valid presentation is counted separately. Presentations during the final 250 ms drain can complete inputs handled within the measured window.

Raw input, paint and actual-present records buffer until normal exit. Non-Partial reason records likewise buffer and describe disabled, prime, unsupported scroll layer/surface/scene, atlas loss, large changes, required copies, identical skips, live resize, verification disable or allocation/verification fallback. No frame-choice policy is changed. A branch without the unmerged G2 back-off implementation cannot produce a back-off frame; that branch distinction must be retained when reporting counts.

## Hover pixel proof

The existing List hover style was tested through real native window captures. With the isolated `RUSTUI_G0_HOVER_PIXELS=1` test control, the normal mouse handler holds row 2 then row 8. The checker captures both windows and asserts that the old row loses its highlight, the new row gains it, both backgrounds exchange the same RGB values, and their positions/heights match the expected rows at Retina scale. It also asserts the run used Direct mode. The control changes only the scripted test trajectory, not component styling or library behavior.

Reproduce after a pinned release build: `python3 scripts/run_g0_hover_pixels.py`.

| Sample | Before RGB | After RGB | Captured center y (px) |
|---|---|---|---:|
| Row 2 | 46,46,50 | 30,30,30 | 214 |
| Row 8 | 30,30,30 | 46,46,50 | 574 |

Captures are 2400 × 1600, scale 2. [Row 2 highlighted](PX2-hover-row-2.png), [row 8 highlighted](PX2-hover-row-8.png). This proves an on-screen highlight transition, not merely dispatched mouse events; it is not a new visual-design approval.

## PX-2 paired measurement

Measured on 2026-11-21, Apple M1 Max, macOS 27.0. One pinned 1.98.0 release build (`0da88057f57b0031b7ea6e7bcd4ab546cf635eaa`), 1200 × 800 points at scale 2; normal warmup followed by 60 seconds per capture. Both arms: `RUSTUI_DAMAGE=0 RUSTUI_FRAME_LOG=buffered RUSTUI_FRAME_PROFILE=1`; candidate off/on: `RUSTUI_G0_LATENCY=0/1`. Pair order was off→on, on→off, off→on, on→off, off→on. No compilation, profiling or other workload ran during timing; the unrelated indexer was paused. Normal macOS services remained active. Raw logs are local ignored artifacts, not committed.

The deciding metric is getrusage user + system CPU seconds per attempted present. Verdicts use unrounded paired changes and the [owner rule](MEASUREMENT_RULES.md). These measure diagnostic overhead, not an application optimization. Negative is less process CPU.

| Scene | Median off ms/attempt | Median on ms/attempt | Five paired changes % | Median % | Min % | Max % | Verdict |
|---|---:|---:|---|---:|---:|---:|---|
| typing | 13.922835 | 13.782783 | +0.978, -1.803, +0.749, -1.096, -1.100 | -1.096 | -1.803 | +0.978 | **NEUTRAL** |
| hover | 2.685403 | 2.698962 | -1.355, +0.594, -0.163, -0.162, +0.407 | -0.162 | -1.355 | +0.594 | **NEUTRAL** |
| log | 3.875103 | 3.812184 | -3.467, +38.647, +1.502, -1.254, -1.624 | -1.254 | -3.467 | +38.647 | **INCONCLUSIVE** |
| drag | 1.469820 | 1.529079 | +0.708, -13.713, +3.743, +6.056, +3.689 | +3.689 | -13.713 | +6.056 | **INCONCLUSIVE** |

### Per-capture process and elapsed-phase diagnostics

CPU seconds are measured-window deltas after warmup. `Attempt` is the measured CPU present-record count. The phase metric is draw + present elapsed milliseconds per attempt, not the process-CPU verdict metric. Slow CPU frames count draw + present >8.3 ms; this is not a display-interval or input-delay count.

| Scene | Pair | Arm | User s | System s | Total CPU s | Attempts | CPU s/attempt | Draw+present ms/attempt | CPU frames >8.3 ms |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| typing | 1 | off | 6.611854 | 0.519876 | 7.131730 | 518 | 0.013767819 | 0.833294 | 0 |
| typing | 1 | on | 6.687749 | 0.499833 | 7.187582 | 517 | 0.013902480 | 0.835198 | 0 |
| typing | 2 | off | 6.713784 | 0.547432 | 7.261216 | 517 | 0.014044905 | 0.819181 | 0 |
| typing | 2 | on | 6.638576 | 0.505488 | 7.144064 | 518 | 0.013791629 | 0.826511 | 0 |
| typing | 3 | off | 6.559726 | 0.539982 | 7.099708 | 519 | 0.013679592 | 0.828303 | 0 |
| typing | 3 | on | 6.626218 | 0.526675 | 7.152893 | 519 | 0.013782067 | 0.833318 | 0 |
| typing | 4 | off | 6.711658 | 0.528216 | 7.239874 | 520 | 0.013922835 | 0.855252 | 0 |
| typing | 4 | on | 6.657115 | 0.503413 | 7.160528 | 520 | 0.013770246 | 0.818806 | 0 |
| typing | 5 | off | 6.705910 | 0.540868 | 7.246778 | 520 | 0.013936112 | 0.864237 | 0 |
| typing | 5 | on | 6.665136 | 0.501911 | 7.167047 | 520 | 0.013782783 | 0.830123 | 0 |
| hover | 1 | off | 13.970740 | 4.607118 | 18.577858 | 7198 | 0.002580975 | 1.716405 | 0 |
| hover | 1 | on | 13.749259 | 4.574252 | 18.323511 | 7197 | 0.002545993 | 1.687419 | 0 |
| hover | 2 | off | 14.484558 | 4.847659 | 19.332217 | 7199 | 0.002685403 | 1.800192 | 0 |
| hover | 2 | on | 14.626936 | 4.812020 | 19.438956 | 7196 | 0.002701356 | 1.807625 | 0 |
| hover | 3 | off | 14.384720 | 4.802496 | 19.187216 | 7195 | 0.002666743 | 1.788058 | 0 |
| hover | 3 | on | 14.305836 | 4.842201 | 19.148037 | 7192 | 0.002662408 | 1.754459 | 0 |
| hover | 4 | off | 14.701171 | 4.893714 | 19.594885 | 7198 | 0.002722268 | 1.823603 | 1 |
| hover | 4 | on | 14.669598 | 4.896342 | 19.565940 | 7199 | 0.002717869 | 1.806686 | 0 |
| hover | 5 | off | 14.539695 | 4.779089 | 19.318784 | 7187 | 0.002688018 | 1.803123 | 0 |
| hover | 5 | on | 14.583537 | 4.830100 | 19.413637 | 7193 | 0.002698962 | 1.797605 | 1 |
| log | 1 | off | 1.610254 | 0.660723 | 2.270977 | 575 | 0.003949525 | 2.309343 | 0 |
| log | 1 | on | 1.549638 | 0.642613 | 2.192251 | 575 | 0.003812610 | 2.192687 | 0 |
| log | 2 | off | 1.048568 | 0.433104 | 1.481672 | 575 | 0.002576821 | 1.368356 | 0 |
| log | 2 | on | 1.455577 | 0.598713 | 2.054290 | 575 | 0.003572678 | 2.060178 | 0 |
| log | 3 | off | 1.483564 | 0.625733 | 2.109297 | 576 | 0.003661974 | 2.079014 | 0 |
| log | 3 | on | 1.517059 | 0.620200 | 2.137259 | 575 | 0.003716972 | 2.162137 | 0 |
| log | 4 | off | 1.620737 | 0.671067 | 2.291804 | 575 | 0.003985746 | 2.341126 | 0 |
| log | 4 | on | 1.602775 | 0.660281 | 2.263056 | 575 | 0.003935750 | 2.288748 | 0 |
| log | 5 | off | 1.578174 | 0.650010 | 2.228184 | 575 | 0.003875103 | 2.269962 | 0 |
| log | 5 | on | 1.551825 | 0.640181 | 2.192006 | 575 | 0.003812184 | 2.207098 | 0 |
| drag | 1 | off | 5.557203 | 5.477121 | 11.034324 | 7194 | 0.001533823 | 0.452310 | 0 |
| drag | 1 | on | 5.605550 | 5.514612 | 11.120162 | 7199 | 0.001544681 | 0.452494 | 0 |
| drag | 2 | off | 5.339872 | 5.238420 | 10.578292 | 7197 | 0.001469820 | 0.429339 | 1 |
| drag | 2 | on | 4.620159 | 4.480909 | 9.101068 | 7176 | 0.001268265 | 0.354772 | 1 |
| drag | 3 | off | 5.454022 | 5.353813 | 10.807835 | 7200 | 0.001501088 | 0.439321 | 0 |
| drag | 3 | on | 5.600777 | 5.611592 | 11.212369 | 7200 | 0.001557273 | 0.455057 | 0 |
| drag | 4 | off | 5.182935 | 5.183400 | 10.366335 | 7190 | 0.001441771 | 0.419417 | 0 |
| drag | 4 | on | 5.471793 | 5.537577 | 11.009370 | 7200 | 0.001529079 | 0.445039 | 1 |
| drag | 5 | off | 5.257183 | 5.301249 | 10.558432 | 7197 | 0.001467060 | 0.425445 | 0 |
| drag | 5 | on | 5.463723 | 5.481140 | 10.944863 | 7195 | 0.001521176 | 0.444462 | 0 |

### Actual display-time diagnostics (candidate on)

Each row reports a separate capture. Median/p95/max apply only to matched inputs with a valid actual `MTLDrawable.presentedTime`. An overwritten state that never painted and a painted state without a valid displayed tag are reported separately. Unavailable timestamps are excluded, never substituted with submission or completion times. These coverage limits prevent treating the measured subset as every dispatched input. Invalid-callback counts include the full capture lifecycle; input/latency counts use the measured window, with its final presentation drain.

| Scene | Pair | Shown / inputs | Median ms | p95 ms | Max ms | Delays >8.3 ms | Coalesced unpainted | Painted, no valid presentation | Unavailable callbacks |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| typing | 1 | 517/600 | 55.995 | 69.491 | 72.508 | 517 | 0 | 83 | 1 |
| typing | 2 | 518/600 | 57.390 | 69.732 | 70.861 | 518 | 0 | 82 | 1 |
| typing | 3 | 519/600 | 56.850 | 65.633 | 70.745 | 519 | 0 | 81 | 1 |
| typing | 4 | 520/600 | 57.364 | 67.307 | 70.777 | 520 | 0 | 80 | 1 |
| typing | 5 | 520/600 | 56.652 | 67.642 | 70.756 | 520 | 0 | 80 | 1 |
| hover | 1 | 3616/7200 | 46.419 | 51.929 | 52.899 | 3616 | 1385 | 2199 | 3593 |
| hover | 2 | 3625/7200 | 46.183 | 51.857 | 57.721 | 3625 | 1293 | 2282 | 3585 |
| hover | 3 | 3631/7200 | 46.622 | 51.965 | 55.278 | 3631 | 1439 | 2130 | 3575 |
| hover | 4 | 3629/7200 | 45.919 | 51.110 | 53.993 | 3629 | 1301 | 2270 | 3583 |
| hover | 5 | 3617/7200 | 45.896 | 51.187 | 57.536 | 3617 | 1364 | 2219 | 3589 |
| log | 1 | 600/600 | 49.844 | 58.154 | 162.145 | 600 | 0 | 0 | 1 |
| log | 2 | 600/600 | 49.878 | 58.192 | 158.234 | 600 | 0 | 0 | 1 |
| log | 3 | 600/600 | 49.408 | 58.167 | 149.715 | 600 | 0 | 0 | 1 |
| log | 4 | 600/600 | 49.629 | 58.121 | 155.296 | 600 | 0 | 0 | 1 |
| log | 5 | 600/600 | 49.413 | 57.817 | 147.998 | 600 | 0 | 0 | 1 |
| drag | 1 | 3629/7200 | 46.854 | 51.880 | 54.090 | 3629 | 1177 | 2394 | 3581 |
| drag | 2 | 3607/7200 | 47.079 | 51.939 | 61.883 | 3607 | 1373 | 2220 | 3574 |
| drag | 3 | 3608/7200 | 47.032 | 51.945 | 58.182 | 3608 | 1296 | 2296 | 3605 |
| drag | 4 | 3691/7200 | 47.298 | 51.938 | 58.192 | 3691 | 1321 | 2188 | 3518 |
| drag | 5 | 3698/7200 | 47.133 | 51.892 | 57.541 | 3698 | 1161 | 2341 | 3505 |

All paired captures asserted damage-off Direct mode, fixed window geometry and expected input counts (600 typing/log, 7200 hover/drag). The binary SHA-256 was checked unchanged across all captures: `ac205256c7c68e2c900679775abab17e71331ee0d7a87ff5859845a4a36aafbb`.

## Validation

Pinned release build passed. `scripts/check.sh` passed all 1,874 tests: 1,867 passed and 7 ignored across all seven test groups (1,456 + 7 ignored; 10; 86; 3; 40; 272; 0). `./scripts/verify-release.sh source` passed 682 tests. Focused correlation/fixture checks, cumulative candidate reverse/apply checks, and generated-owned-source verification passed. Native hover capture passed on the final build. No visual approval is claimed.

The paired verdicts satisfy the owner rule for retaining a diagnostic ticket: checks are green and no affected scene is REGRESSION. NEUTRAL and INCONCLUSIVE are not gains. PR #163 remains draft; Fix F thresholds are unchanged. G2-8b and P1 are not started.

## Damage-on reason captures

Five separate 60-second measured windows after normal warmup, the same release binary, `RUSTUI_DAMAGE=1 RUSTUI_FRAME_LOG=buffered RUSTUI_FRAME_PROFILE=1 RUSTUI_G0_LATENCY=1`. These are diagnostic counts, not paired CPU verdict captures. Every observed non-Partial frame has a reason record. No mismatch event occurred.

| Scene | Skip | Copy | Partial | Retained | Direct | Non-Partial reasons |
|---|---:|---:|---:|---:|---:|---|
| idle | 0 | 0 | 114 | 0 | 0 | none |
| typing | 80 | 0 | 439 | 0 | 0 | identical: 80 |
| hover | 0 | 59 | 0 | 2569 | 4568 | large: 7137, required: 59 |
| log | 0 | 0 | 0 | 0 | 575 | large: 575 |
| drag | 1 | 1284 | 5903 | 0 | 0 | identical: 1, required: 1284 |

Unobserved reasons are zero in these windows: unsupported scroll layer/surface/scene, prime, atlas loss, live resize and verification/allocation fallback. Prime work was outside the measured windows. **Back-off is absent from this measured branch**, which starts at `g-integration` 22785a44d and adds PX-2 diagnostics. The unmerged E/F implementation remains on `g2-7-damage-cpu-budget` (draft PR #163); its back-off reason counts with buffered logging are **NOT VERIFIED** here. These counts must not be substituted for that branch’s results. Fix F thresholds are unchanged.
