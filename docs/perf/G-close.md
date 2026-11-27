# Stage G · Metal damage tracking close-out

**Shipping status: OPT-IN.** For a GPU-bound card drag, enabling damage reduced GPU time by 38% and process CPU was NEUTRAL in all five pairs. The draw+present gate for drag remains INCONCLUSIVE. For typing and hover, the CPU evidence does not support enabling damage. Scroll still draws the whole window. No workload met the process-CPU GAIN rule. Power was outside this ticket.

## Final qualified workload results

RustUI Lab used a 3456 × 2234 drawable, the G5 profile, display sync on, native input and buffered logging. Values below are damage off → on. GPU time, draw+present and process CPU are the medians of five arm means. Percent Partial and planner time pool measured attempts. The historical workload gate evaluates each pair against 3% draw+present overhead, except scroll, which uses a 2% process-CPU limit. The P verdict uses the five paired changes in process CPU seconds per second.

| Workload | Qualified | Partial % | GPU ms/frame | Draw+present ms/attempt | Mean plan µs | Process CPU s/s | Workload gate | Process CPU verdict |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Typing | 5/5 | 0 → 100 | 1.032 → 0.898 | 1.854 → 2.047 | 7.03 → 56.26 | 0.1004 → 0.1020 | FAIL | INCONCLUSIVE |
| Hover | 5/5 | 0 → 99.82 | 1.324 → 0.935 | 1.979 → 2.045 | 6.02 → 81.85 | 0.3544 → 0.3729 | INCONCLUSIVE | INCONCLUSIVE |
| Drag | 5/5 | 0 → 99.997 | 1.903 → 1.187 | 0.609 → 0.652 | 9.03 → 41.08 | 0.2825 → 0.2807 | INCONCLUSIVE | NEUTRAL |
| Scroll | 5/5 | 0 → 0 | 1.713 → 1.711 | 1.070 → 1.130 | 9.12 → 9.89 | 0.3435 → 0.3494 | INCONCLUSIVE | NEUTRAL |

### Drag qualification

The Lab and input helpers matched the saved G-R2e build by SHA-256 before and after the captures. Pair 1 carries forward. Pairs 2–5 completed on the same binary before adding G-R3 instrumentation. Odd pairs ran off first, even pairs on first. Each arm lasted 60 seconds after warmup. Both arms had to average at most 100% of one core in other processes, with an absolute difference of at most 15 percentage points. The Lab, WindowServer and kernel_task were excluded from qualification. Each qualified attempt reported no thermal or performance warning and no CPU limit.

| Pair | Accepted attempt | Background off/on, % | WindowServer off/on, % | kernel_task off/on, % | Process CPU change | Draw+present change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 1 | 23.2 / 26.1 | 48.3 / 48.3 | 11.7 / 11.7 | +0.01% | +2.83% |
| 2 | 4 | 24.1 / 21.4 | 40.0 / 45.0 | 13.3 / 15.0 | −0.62% | +7.10% |
| 3 | 1 | 25.3 / 20.7 | 40.0 / 45.0 | 13.3 / 13.3 | −2.38% | +3.61% |
| 4 | 2 | 35.5 / 29.0 | 41.7 / 45.0 | 15.0 / 13.3 | −1.27% | +6.77% |
| 5 | 1 | 21.8 / 19.6 | 41.7 / 45.0 | 13.3 / 13.3 | −0.36% | +8.22% |

Pair 2 qualified on its third and final retry. Its discarded background readings were 137.9/26.7, 103.9/103.8 and 24.0/53.7. Pair 4 discarded 37.7/20.2. Damage-on drag submitted 29,976 Partial frames and one Copy. Four of the five pairs exceeded the 3% draw+present threshold.

## Where typing spends draw+present time

**The historical counter that explains the residual is other host work, concentrated in `Window::draw`.** Pair 4 supplied both published G-R2e draw+present medians. Its raw records reconcile exactly to 1,854.5 → 2,047.2 µs per attempt. Of the +192.8 µs, planning accounted for +49.3 µs and other accounted for +121.4 µs, or 85% of the remaining +143.5 µs. Encoding, including the copy, accounted for +18.4 µs and present, including acquisition, for +3.7 µs. Within the draw, paint rose 34.5 µs, prepaint rose 33.0 µs, and time outside the original draw-phase markers rose 49.5 µs. Those records do not isolate a single function within the last amount.

The following independent, unqualified 60-second diagnostic pair used the full Lab typing scene, with 583 off and 586 on attempts. Each had exactly one renderer submission. Times are elapsed host µs per attempted present, including time blocked on a drawable. Process CPU verdicts above use getrusage. The six components sum to the mean total before rounding. Each p95 is a nearest-rank percentile of that component, so percentiles need not add.

| Host component | Off mean / p95 µs | On mean / p95 µs | Mean change µs |
| --- | ---: | ---: | ---: |
| Plan | 8.52 / 16.58 | 54.91 / 90.88 | +46.39 |
| Encode, excluding retained copy | 198.28 / 413.83 | 192.59 / 403.54 | −5.69 |
| Encode retained-to-drawable copy | 0 / 0 | 18.95 / 33.63 | +18.95 |
| Present, excluding drawable wait | 49.04 / 92.71 | 49.24 / 92.04 | +0.20 |
| Drawable wait (`nextDrawable`) | 68.36 / 134.63 | 68.60 / 135.54 | +0.24 |
| Other, including CPU scene construction | 1,655.34 / 2,646.08 | 1,650.59 / 2,647.79 | −4.75 |
| **Total draw+present** | **1,979.54 / 3,214.17** | **2,034.88 / 3,343.33** | **+55.34** |

This pair reproduced +8.95 µs of non-planner overhead. The new marker around `self.next_frame.finish()` rose 43.47 → 83.15 µs, while build and prepaint fell by 14.22 and 22.69 µs. The historical 144 µs is not a fixed per-keystroke copy charge.

## What changed on screen

The Metal editor oracle rendered the G0 10,000-line source document at 3456 × 2234, with an 1136 × 700 logical-point editor. Each operation ran 60 times at the same location and restored the starting state between trials. The 300 frames in the table exclude the additional 60 newline removals and 60 paste undos, which also passed verification. Each frame asserted Partial, started the partial pass with magenta, checked declared coverage, and compared its output byte for byte to an independent full draw. All 420 frames passed.

Percentages use the entire drawable. Declared area is the union of clipped changed line declarations, old/new caret and selection rectangles, and any explicit reflow region after a one-device-pixel AA margin. Changed area counts distinct pixels whose four bytes differ in consecutive full references. The actual scissor is the single bounding rectangle the planner submitted. It can be wider than the declared union after ordinary signature matching, or smaller when declared content is unchanged. A mean and p95 agree here because every trial of a given operation made the same edit at the same position.

| Edit | Frames | Declared % mean / p95 | Changed pixels % mean / p95 | Actual scissor % mean / p95 |
| --- | ---: | ---: | ---: | ---: |
| Insert character | 60 | 1.5603 / 1.5603 | 0.04184 / 0.04184 | 3.1193 / 3.1193 |
| Delete character | 60 | 1.5603 / 1.5603 | 0.04184 / 0.04184 | 3.1193 / 3.1193 |
| Newline | 60 | 44.4254 / 44.4254 | 0.47702 / 0.47702 | 41.1983 / 41.1983 |
| Multiline paste | 60 | 44.4254 / 44.4254 | 0.48433 / 0.48433 | 41.1983 / 41.1983 |
| Caret blink | 60 | 0.00249 / 0.00249 | 0.00114 / 0.00114 | 0.00389 / 0.00389 |

The reporting fixture explicitly enables `gpui_platform/font-kit`. A reference-image assertion checks for dark document ink away from the caret, gutter and borders. The original test fixture selected GPUI’s no-op text system, so it could pass image equality while omitting glyphs in both images. Its area measurements were discarded. The table describes the controlled editor operations. The following capture measures the complete Lab window.

### Why typing saved 13% GPU time instead of 53%

**Editor declarations enlarged the single Partial scissor from 0.132% to 39.207% of the Lab window on average, forcing more of the existing scene to be redrawn before the same full-window copy.** Five same-build diagnostic pairs reproduced damage-on GPU times of 0.491 ms/frame with declarations off and 0.895 ms/frame with declarations on. The published G-R2d/G-R2e values were 0.490/0.898, with near-identical Direct baselines of 1.037/1.032. The G-R2e fast planner includes unmatched primitive spans per stream, checks a fixed area threshold, and submits one bounding rectangle. The renderer executes that rectangle even when most pixels within it are unchanged.

| Native Lab, damage on | Pair 1 | Pair 2 | Pair 3 | Pair 4 | Pair 5 | Partial scissor mean / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Declarations off, GPU ms/frame | 0.483 | 0.495 | 0.499 | 0.491 | 0.487 | 0.132% / 0.132% |
| Declarations on, GPU ms/frame | 0.933 | 0.922 | 0.874 | 0.895 | 0.881 | 39.207% / 40.096% |

The arms alternated order, with five 60-second captures per arm and no GPU counter sampling. The scissor distribution covers 2,843 Partial frames with declarations off and 2,927 with declarations on. The 76 Copy-only frames in the former arm took **278.84 µs mean / 296.88 µs p95** for their entire GPU command, providing a separate reference for the full-screen copy in isolation.

### Cost of the retained copy on Partial frames

Metal sampled the start and end of the copy pass’s vertex and fragment stages on the original command buffer. Each sample below belongs to a Partial frame. The measured timestamp rate was 1.000 GHz in both captures, including when estimated separately on each half. Every fragment interval fit inside the corresponding pass span, and every pass span fit inside its command buffer.

| Declarations | Partial samples | Copy fragment GPU µs mean / p95 | Copy pass first stage to last, µs mean / p95 | Whole command GPU µs mean / p95 |
| --- | ---: | ---: | ---: | ---: |
| Off | 563 | 160.55 / 172.25 | 323.74 / 348.63 | 485.45 / 508.38 |
| On | 585 | **166.12 / 176.29** | **758.54 / 786.00** | 911.74 / 948.21 |

The fragment stage reads the kept texture and writes the drawable. It took 5.57 µs longer with declarations on. The 758.54 µs first-to-last span also includes the time between the vertex and fragment samples, during which other GPU work can execute. Charging that entire span to copying pixels would overstate the saving available from eliminating the copy. The separately measured Copy-only command above gives a second reference.

A supplemental 35-second fullscreen Lab verification run completed 350 native input ticks. The renderer recorded `allocation_or_verify` during warmup followed by `verify_off_path`; it verified zero measured Partial frames. The old capture reader also rejected the untimestamped `declared_full_diff` record. This capture contributes no pixel evidence. The 300-frame table above uses successful, mode-asserting, full-size oracle captures with visible glyphs.

## Backlog

| Follow-up | Size | Evidence and scope |
| --- | --- | --- |
| Tighten editor rectangles | S | Insert declared 1.56% for 0.042% changed. Newline and paste declared 44.43% for under 0.49% changed. Keep the full-reference check for glyph overhangs, caret, selection and reflow. Verify the single scissor also gets smaller in the full Lab scene. |
| Reduce non-planner typing CPU | Only if the increase reappears | The historical +143.5 µs residual included +121.6 µs in `Window::draw`. The new pair measured +39.7 µs in `next_frame.finish()` and +19.0 µs to encode the copy. Reproduce the scene-build delta across pairs before changing the implementation. |
| Avoid the full retained-to-drawable copy | M/L | Every Partial frame still copies all 7.72 million drawable pixels. The copy fragment averaged 166 µs, and a Copy-only command averaged 279 µs. A persistent presentation texture or IOSurface-backed layer would need separate proofs for ownership, in-flight frames, alpha, resize and presentation order. |
| Metal scroll shift | M | All 5,289 historical shift plans returned `shift_unimplemented`; the current early rejection kept every qualified scroll frame Direct. A retained-to-scratch-to-retained blit and exposed-strip redraw need a byte oracle for overlapping moves and a fractional-scroll fallback. |
| Fullscreen verification budget | Diagnosed in G-FS1 | The fullscreen verification run fell back to Direct during warmup. [G-FS1](G-FS1-fullscreen.md) reproduced the verifier’s reference-texture budget refusal. Enabling damage by default or combining it with fullscreen direct-to-display remains an owner decision. |

## Verification and reproduction

`scripts/check.sh` ran seven library test groups on Rust 1.98.0: **1,875 passed, 7 ignored, 0 failed**. All nine native Metal proofs passed, including the two intentional mismatch injections. Both candidate stacks verified exact reverse and forward restoration, and candidate-mode source verification passed.

To reproduce the pixel table, point the Lab’s `RUSTUI_G5_L7_EXPORT_DOCUMENT` at an output file when capturing typing. Then run:

```sh
python3 scripts/report_g_r3.py capture-area /path/to/exported-document.rs target/headstart/g-r3-repro
```

For the GPU copy measurement, enable `RUSTUI_G_R3_COPY_GPU=1` for a buffered, damage-on Partial typing capture. Run `python3 scripts/report_g_r3.py copy /path/to/capture.log` to check the clock rate, the number of Partial samples and the per-command duration bound. `RUSTUI_G_R3_CPU=1 RUSTUI_G_R2_ATTRIBUTION=1` enables the separate CPU probe. Run `python3 scripts/report_g_r3.py cpu /path/to/arm` for each capture directory. The GPU counter sampler was disabled for the CPU pair.

Earlier measurements and decisions: [G-R1 hover diagnosis](G-R1-hover-diagnosis.md), [G-R2 damage engagement and G-R2d results](G-R2-damage-engagement.md), [G-R2e editor-declared damage](G-R2e-editor-damage.md).
