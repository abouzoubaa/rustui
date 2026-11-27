# Metal scroll shift

G3 adds an opt-in scroll path on top of G-FS2, merged into `g5-integration` at `d6c88af389f05fd849740d88af9f1a8cc3890f8f`. Set both `RUSTUI_DAMAGE=1` and `RUSTUI_DAMAGE_SCROLL_SHIFT=1` to enable it. For byte comparisons, use `RUSTUI_DAMAGE=verify` with the same scroll switch. Both switches default to off.

## Gate result

Both workloads reached GPU GAIN and process CPU REGRESSION. The G3 shipping gate failed at both sizes. Damage and scroll shift remain opt-in.

| Drawable | Qualified pairs | Shift frames off/on | GPU ms/frame off/on | Draw + present ms/attempt off/on | Process CPU s/s off/on | GPU / process CPU |
|---|---:|---:|---:|---:|---:|---|
| Windowed 2400 × 1600 | 4/5 | 0 / 99.992% | 0.6226 / 0.5112 | 0.7620 / 0.9769 | 0.2411 / 0.2602 | GAIN / REGRESSION |
| Native fullscreen 3456 × 2168 | 5/5 | 0 / 100.000% | 0.7834 / 0.6358 | 0.9527 / 1.2068 | 0.3220 / 0.3378 | GAIN / REGRESSION |

Times in the summary are medians of qualified arm means. Shift share pools submitted frames. Windowed pair 4 exceeded the other-load limits on all four attempts. The full pair and load tables follow below.

## Rendering

Each surface records the clip and actual painted offset of each scroll container in device pixels. Its two private BGRA8Unorm textures alternate as source and destination on submitted retained frames. A single Metal command buffer copies stationary areas without translation, translates the intersection inside each moving clip, repairs exposed strips and other changed pixels in scene paint order, and copies the result to the drawable. The source and destination are always distinct. GPU completion releases the per-window budget reservations when textures retire.

Fractional or oversized motion redraws the full affected clip. Overlapping moving clips and translucent overlays fall back to Direct. Disjoint moving clips can shift independently in both axes. Geometry changes and texture loss require a full prime before the next shift.

An initial 10-second Lab probe uploaded a mean of 1,662 instances per damage-on frame for a scene averaging 653 instances. Small disconnected glyph repairs along a single edge caused repeated uploads. The final renderer joins nearby repairs on the same scanlines and uploads only intersecting quads and glyphs for simple lists. Complex scenes use the ordinary renderer under the repair scissor.

## Per-surface memory

These are Metal's padded allocation sizes, measured on the native device. Both allocations count against the shared window budget. The verifier's reference texture is separate. Multiple composition surfaces share the 512 MiB hard ceiling.

| Drawable | A (bytes) | B (bytes) | Combined charge (bytes) | Limit with 10% margin (bytes) |
|---|---:|---:|---:|---:|
| 3456 × 2168 | 30,605,312 | 30,605,312 | 61,210,624 | 67,331,687 |
| 5120 × 2880 | 60,030,976 | 60,030,976 | 120,061,952 | 132,068,148 |
| 6016 × 3384 | 82,673,664 | 82,673,664 | 165,347,328 | 181,882,061 |

## Pixel checks

Each row starts with one separately verified prime. The next 300 frames compare the actual GPU output with an independent full redraw, byte for byte. The test asserts the mode and reason for every frame and prefills repair regions with magenta. The slow-scroll cases also check that most of the original clip actually survives the copy.

| Case | Shift frames / 300 | Fallback reasons after prime | Pixel mismatches |
|---|---:|---|---:|
| Slow scroll down | 300 (100%) | None | 0 |
| Slow scroll up | 300 (100%) | None | 0 |
| Momentum, variable step | 300 (100%) | None | 0 |
| Fractional motion at 2× | 0 | 300 `fractional_delta`, clip redraw | 0 |
| Opaque sticky header | 300 (100%) | None | 0 |
| Visible moving scrollbar | 300 (100%) | None | 0 |
| Nested containers moving together | 0 | 300 `nested_overlap`, Direct | 0 |
| Hover and independent external change | 300 (100%) | None | 0 |
| Append-only log at the bottom | 300 (100%) | None | 0 |
| Disjoint horizontal and vertical scroll | 300 (100%) | None | 0 |
| Motion exceeding the clip extent | 0 | 300 `delta_too_large`, clip redraw | 0 |
| Translucent overlay above the container | 0 | 300 `overlay_translucent`, Direct | 0 |
| Translucent sticky decoration inside the container | 0 | 300 `overlay_translucent`, Direct | 0 |

Two additional live verify captures used native Lab scroll input on the frozen release build. Windowed verified 207 shifted outputs and fullscreen verified 119. Both captures completed with zero pixel mismatches. A separate transition proof verified that resize and explicit texture loss each prime once and then resume Shift.

## Paired captures

The two geometries use the same release build, native Cocoa input, buffered logs, the G5 profile and display sync. Each pair alternates which arm runs first. Each arm measures 60 seconds after warmup. Qualification uses measured-window background CPU. It excludes the Lab, WindowServer and kernel_task, limits each arm to 100% of one core, and limits the arm difference to 15 percentage points. A CPU speed limit below 100% causes a thermal retry. Each pair gets up to three retries. Four qualified pairs are required for a verdict.

GPU time is per submitted frame. Process CPU is the Lab's user plus system CPU time per second. Draw + present time per attempt is diagnostic. Verdicts use unrounded paired changes. The gate requires a GPU GAIN and a process CPU verdict other than REGRESSION.

### Windowed

| Pair (try) | Shift % off/on | GPU ms/frame off/on | GPU Δ | Draw + present ms/attempt off/on | Process CPU s/s off/on | CPU Δ |
|---|---:|---:|---:|---:|---:|---:|
| 1 (1) | 0.00 / 100.00 | 0.6251 / 0.5123 | -18.04% | 0.7681 / 0.9771 | 0.2416 / 0.2603 | +7.78% |
| 2 (2) | 0.00 / 99.98 | 0.6213 / 0.5142 | -17.23% | 0.7553 / 0.9767 | 0.2406 / 0.2600 | +8.08% |
| 3 (2) | 0.00 / 99.98 | 0.6218 / 0.5059 | -18.65% | 0.7560 / 0.9665 | 0.2397 / 0.2564 | +6.96% |
| 5 (1) | 0.00 / 100.00 | 0.6234 / 0.5100 | -18.19% | 0.7799 / 0.9930 | 0.2488 / 0.2610 | +4.91% |

Qualified 4/5. GPU GAIN, process CPU REGRESSION. Gate **FAIL**. Damage off submitted 24,784 Direct frames. Damage on submitted 24,681 Shift and 2 Copy frames.

| Pair / try | Qualified | Other CPU % off/on | WindowServer % off/on | kernel_task % off/on | Thermal CPU speed limit |
|---|---|---:|---:|---:|---|
| 1 / 1 | Yes | 90.28 / 89.66 | 46.67 / 46.66 | 16.67 / 15.00 | No limit reported |
| 2 / 1 | No | 115.60 / 101.12 | 46.67 / 44.99 | 18.33 / 16.66 | No limit reported |
| 2 / 2 | Yes | 94.28 / 88.99 | 44.99 / 46.66 | 15.00 / 16.67 | No limit reported |
| 3 / 1 | No | 92.93 / 117.35 | 46.66 / 46.66 | 15.00 / 16.66 | No limit reported |
| 3 / 2 | Yes | 88.32 / 98.26 | 46.67 / 46.66 | 16.67 / 15.00 | No limit reported |
| 4 / 1 | No | 192.45 / 138.69 | 46.67 / 45.00 | 16.67 / 16.67 | No limit reported |
| 4 / 2 | No | 161.26 / 121.62 | 45.00 / 46.66 | 16.67 / 20.00 | No limit reported |
| 4 / 3 | No | 189.13 / 168.51 | 45.00 / 46.66 | 18.33 / 16.66 | No limit reported |
| 4 / 4 | No | 148.25 / 175.28 | 46.66 / 46.67 | 18.33 / 18.33 | No limit reported |
| 5 / 1 | Yes | 96.60 / 95.19 | 44.99 / 46.66 | 15.00 / 16.67 | No limit reported |

### Fullscreen

| Pair (try) | Shift % off/on | GPU ms/frame off/on | GPU Δ | Draw + present ms/attempt off/on | Process CPU s/s off/on | CPU Δ |
|---|---:|---:|---:|---:|---:|---:|
| 1 (1) | 0.00 / 100.00 | 0.7814 / 0.6388 | -18.25% | 0.9527 / 1.2054 | 0.3247 / 0.3364 | +3.62% |
| 2 (1) | 0.00 / 100.00 | 0.7834 / 0.6337 | -19.12% | 0.9374 / 1.2211 | 0.3220 / 0.3408 | +5.85% |
| 3 (1) | 0.00 / 100.00 | 0.7788 / 0.6398 | -17.85% | 0.9758 / 1.2068 | 0.3275 / 0.3378 | +3.14% |
| 4 (2) | 0.00 / 100.00 | 0.7864 / 0.6358 | -19.15% | 0.9137 / 1.2413 | 0.3104 / 0.3459 | +11.46% |
| 5 (1) | 0.00 / 100.00 | 0.7854 / 0.6339 | -19.29% | 0.9868 / 1.1771 | 0.3106 / 0.3274 | +5.43% |

Qualified 5/5. GPU GAIN, process CPU REGRESSION. Gate **FAIL**. Damage off submitted 30,737 Direct frames. Damage on submitted 31,081 Shift frames.

| Pair / try | Qualified | Other CPU % off/on | WindowServer % off/on | kernel_task % off/on | Thermal CPU speed limit |
|---|---|---:|---:|---:|---|
| 1 / 1 | Yes | 51.10 / 52.20 | 36.67 / 38.33 | 10.00 / 11.67 | No limit reported |
| 2 / 1 | Yes | 49.04 / 48.37 | 38.33 / 39.99 | 10.00 / 11.66 | No limit reported |
| 3 / 1 | Yes | 39.17 / 49.45 | 36.66 / 38.33 | 10.00 / 10.00 | No limit reported |
| 4 / 1 | No | 52.57 / 72.77 | 38.33 / 38.33 | 10.00 / 10.00 | No limit reported |
| 4 / 2 | Yes | 55.48 / 54.38 | 38.33 / 39.99 | 11.67 / 11.66 | No limit reported |
| 5 / 1 | Yes | 57.24 / 71.74 | 39.99 / 43.33 | 11.67 / 13.33 | No limit reported |


## Where the additional time appeared

The following means pool the existing per-frame counters over qualified measured windows. They measure elapsed time in renderer phases. The process CPU verdict above uses the Lab’s own user and system CPU time.

| Drawable | Plan µs off/on | Encode µs off/on | Present µs off/on | Instances encoded off/on |
|---|---:|---:|---:|---:|
| Windowed | 7.9 / 137.5 | 87.4 / 157.0 | 92.2 / 86.0 | 653.4 / 28.9 |
| Native fullscreen | 7.8 / 188.7 | 123.1 / 203.9 | 112.2 / 92.3 | 653.4 / 28.7 |

Planning showed the largest increase at both sizes. The windowed damage-on signature collection and scroll comparison averaged 26.8 and 100.5 µs respectively. Fullscreen they averaged 36.9 and 142.4 µs. Encoding also took longer despite uploading fewer instances.

## Verification

The repository-pinned Rust 1.98.0 check exited successfully across seven test groups with 1,875 passed, 7 ignored and 0 failed. Both Fast stacks passed exact reverse and replay. Both owned-source checks and candidate-mode source verification passed. The nine existing native Metal proofs passed alongside the scroll pixel histories and size checks.

## Reproduction

Run from the repository root with a fresh output directory for each matrix.

```sh
scripts/run_g3_metal_proofs.sh
bash scripts/build_g5_refs.sh
export RUSTUI_DAMAGE_SCROLL_SHIFT=1 RUSTUI_G_R2_VIEW_SIGNATURES=1 RUSTUI_DAMAGE_DECLARED=1
RUSTUI_G_FS1_SEQUENCE=C python3 scripts/run_g_r2.py matrix --scene scroll --output target/headstart/g3-windowed
RUSTUI_G_FS1_SEQUENCE=A python3 scripts/run_g_r2.py matrix --scene scroll --output target/headstart/g3-fullscreen
```
