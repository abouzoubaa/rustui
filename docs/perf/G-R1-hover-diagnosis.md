# G-R1 hover diagnosis and full-display comparison

G-R1 starts from the G-R0 merge into `g5-integration`. The planner probe
and Lab's borderless fullscreen mode are opt-in. They leave the renderer's
decision rules and all defaults unchanged.

## Method

Use the frozen release produced by `bash scripts/build_g5_refs.sh` with
pinned Rust 1.98.0. The timing workload keeps the same Lab scene content,
native Cocoa events, input rates, direct launch, ScreenCaptureKit version
marker, two-second warmup, buffered logging, and 60-second measured
window as G-R0. Use `RUSTUI_G5_PROFILE=1`, display sync on, and planner
back-off off. G-R0 ran at 2400×1600 device pixels.

The two diagnosis captures use damage on at the original window size
and `RUSTUI_G_R1_TRACE=1`. The planner records a reason at each
comparable frame decision and records changed primitive rectangles on
every 32nd decision. The normal frame logs also count preflight
rejections, such as unsupported scroll tiles, and submitted GPU modes.
The general planner's coarse scroll buckets can reject individual
matches. The final decision still records its actual branch, such as
summed area, invalid scroll geometry, or conflicting order.

The size comparison uses `RUSTUI_G_R1_FULLSCREEN=1` to make the Lab
borderless across the built-in display, including the area beside the
notch. Both arms have planner tracing off. Run five matched damage
off/on pairs per scene and alternate which arm starts. Verify the logged
viewport times backing scale in every capture. The process CPU verdict
uses the P rule per attempted present, matching G-R0. The report also
lists process CPU per wall second and the draw+present gate separately.
The frozen build includes the ScreenCaptureKit observer and native input
helper. The observer waits for fullscreen geometry before starting. The
short hover and scroll probes agreed on 3456×2234 from the Lab viewport
and the actual Metal planner area.

Reproduce the full-display pairs with the G-R0 runner.

```sh
RUSTUI_G_R1_FULLSCREEN=1 RUSTUI_G_R1_TRACE=0 \
  python3 scripts/run_g_r0.py matrix --scene typing --scene hover --scene drag \
  --scene scroll --output target/headstart/g-r1/reproduce
```

The workspace check passed 1,875 tests across seven groups with seven
ignored and zero failures. Candidate source verification, both Fast
stack checks, and nine native Metal damage proofs passed.

## Hover planner decisions

**Hover never uses Partial because every comparable hover frame reverses
a pair of previously ordered primitive groups (2→3 and 3→2) and the
planner rejects that ordering before it computes damage area.**

The 60-second trace recorded 6,241 Direct frames and zero Partial frames.
All 6,241 rejected plans went through the general planner’s order check.
Area, coarse matching, unsupported tiles, shift and other reasons had
zero rejections. This capture was measured on a loaded machine
(6.38% background at start, 9.19% at end).

These are 20 equally spaced detail samples from the measured window.
Each position is the top-left corner in device pixels. Rows are the Lab
labels’ zero-based indices. Each listed row rectangle is a highlight
quad. The listed barcode positions each describe an 8×64 rectangle at
`y=20` that changed colour as the input version advanced. The order
check exits before the planner merges these candidate rectangles.

| Trace frame | Highlight removed, row and rectangle | Highlight added, row and rectangle | Changed barcode rects, x |
| ---: | --- | --- | --- |
| 0 | `00011` (64, 724) 2272×60 | `00001` (64, 124) 2272×60 | 2192 |
| 320 | `00016` (64, 1024) 2272×60 | `00015` (64, 964) 2272×60 | 2192 |
| 640 | `00010` (64, 664) 2272×60 | `00011` (64, 724) 2272×60 | 2192 |
| 960 | `00005` (64, 364) 2272×60 | `00004` (64, 304) 2272×60 | 2184, 2192 |
| 1312 | `00011` (64, 724) 2272×60 | `00012` (64, 784) 2272×60 | 2184, 2192 |
| 1632 | `00005` (64, 364) 2272×60 | `00004` (64, 304) 2272×60 | 2184, 2192 |
| 1952 | `00021` (64, 1324) 2272×60 | `00020` (64, 1264) 2272×60 | 2184, 2192 |
| 2272 | `00005` (64, 364) 2272×60 | `00006` (64, 424) 2272×60 | 2184, 2192 |
| 2624 | `00020` (64, 1264) 2272×60 | `00019` (64, 1204) 2272×60 | 2192 |
| 2944 | `00006` (64, 424) 2272×60 | `00007` (64, 484) 2272×60 | 2192 |
| 3264 | `00010` (64, 664) 2272×60 | `00009` (64, 604) 2272×60 | 2192 |
| 3584 | `00016` (64, 1024) 2272×60 | `00017` (64, 1084) 2272×60 | 2192 |
| 3936 | `00010` (64, 664) 2272×60 | `00009` (64, 604) 2272×60 | 2192 |
| 4256 | `00016` (64, 1024) 2272×60 | `00017` (64, 1084) 2272×60 | 2192 |
| 4576 | `00000` (64, 64) 2272×60 | `00001` (64, 124) 2272×60 | 2192 |
| 4896 | `00016` (64, 1024) 2272×60 | `00015` (64, 964) 2272×60 | 2192 |
| 5248 | `00000` (64, 64) 2272×60 | `00001` (64, 124) 2272×60 | 2192 |
| 5568 | `00016` (64, 1024) 2272×60 | `00015` (64, 964) 2272×60 | 2192 |
| 5888 | `00010` (64, 664) 2272×60 | `00011` (64, 724) 2272×60 | 2192 |
| 6240 | `00016` (64, 1024) 2272×60 | `00015` (64, 964) 2272×60 | 2192 |

## Scroll planner decisions

All 6,349 scroll attempts drew Direct at 2400×1600. The diagnosis split
the old `large` label into two causes.

| Reason | Hover | Scroll |
| --- | ---: | ---: |
| Reversed paint order | 6,241 | 0 |
| Summed rectangles over the 60% area limit | 0 | 1,060 |
| Single bounding box over the 60% area limit | 0 | 0 |
| Accepted scroll shift, Metal shift path unimplemented | 0 | 5,289 |
| Rejected coarse candidates | 0 | 0 |
| All other fallbacks | 0 | 0 |

The planner found an integer vertical move in 5,289 frames (83.3%).
Metal rejects such plans even if their damage is small. The median
combined dirty area for these plans was 5.73% of the window
(2.31–8.37%). The other 1,060 frames exceeded the planner’s cap after
the damage merged into a 2272×1516 rectangle at (64, 20), covering 89.70%
of the window.

For example, scroll trace 64 had a 2144×66 rectangle at (64, 20)
and a 2272×32 rectangle at (64, 1504). The bounding box of those two
rectangles covers 89.70% of the window. A shift implementation that
redraws their bounding box would lose most of the benefit. The scroll
scene used no tiled layers on this run. Its capture was measured on a
loaded machine (3.29% background at start, 5.64% at end).

### Version marker control

The same 60-second native workloads at 2400×1600 also ran with the
ScreenCaptureKit version marker disabled. Hover still drew 6,270 Direct
frames, all rejected for reversed paint order. Scroll drew 6,315 Direct
frames. Of those, 5,260 were unimplemented shift plans and 1,055 exceeded
the area limit. Neither workload used Partial. The hover control was
measured on a loaded machine (11.39% background at start, 9.97% at end).
Scroll was quiet (3.05% at start, 4.84% at end).

## Full-display comparison

All 40 captures completed at 3456×2234 device pixels on an Apple M1 Max
running macOS 27.3. The Lab used the
same scene content at both sizes. Values are medians across the five
captures per arm, formatted damage off → on. G-R0 data comes from
[G-R0 rebaseline](G-R0-rebaseline.md).

| Scene | GPU ms/frame 2400×1600 | GPU ms/frame 3456×2234 | Draw+present CPU ms/attempt 2400×1600 | Draw+present CPU ms/attempt 3456×2234 | Stage G CPU gate, small / full |
| --- | ---: | ---: | ---: | ---: | --- |
| Typing | 0.728 → 0.334 | 1.041 → 0.517 | 1.685 → 1.876 | 2.067 → 2.334 | FAIL / FAIL |
| Hover | 0.398 → 0.398 | 0.719 → 0.730 | 1.896 → 1.916 | 2.467 → 2.434 | N/A — damage path not exercised |
| Drag | 0.663 → 0.403 | 0.883 → 0.499 | 0.746 → 0.827 | 1.113 → 1.244 | INCONCLUSIVE / FAIL |
| Scroll | 0.587 → 0.589 | 0.890 → 0.903 | 1.064 → 1.436 | 1.451 → 1.877 | FAIL / FAIL |

| Scene | Process CPU ms/attempt 2400×1600 | Verdict | Process CPU ms/attempt 3456×2234 | Verdict | Full-size process CPU s/s | Verdict |
| --- | ---: | --- | ---: | --- | ---: | --- |
| Typing | 8.429 → 8.623 | INCONCLUSIVE | 10.932 → 11.284 | INCONCLUSIVE | 0.1063 → 0.1096 | INCONCLUSIVE |
| Hover | 3.739 → 3.758 | INCONCLUSIVE | 4.376 → 4.284 | INCONCLUSIVE | 0.4304 → 0.4301 | INCONCLUSIVE |
| Drag | 2.778 → 2.885 | INCONCLUSIVE | 3.949 → 4.073 | INCONCLUSIVE | 0.3926 → 0.4051 | INCONCLUSIVE |
| Scroll | 2.959 → 3.169 | REGRESSION | 3.652 → 3.843 | INCONCLUSIVE | 0.3716 → 0.3940 | REGRESSION |

| Scene | Full-size draw+present changes, pairs 1–5 | Process CPU/attempt changes | Process CPU/s changes |
| --- | --- | --- | --- |
| Typing | +11.94%, +20.48%, +11.86%, +12.11%, +15.34% | +1.87%, +6.04%, +2.79%, +2.44%, +4.16% | +2.20%, +6.83%, +1.61%, +2.18%, +4.80% |
| Hover | +2.56%, -1.18%, +2.58%, -4.84%, +1.77% | +4.04%, -1.03%, +0.60%, -6.61%, +2.03% | +3.67%, -0.07%, +0.89%, -7.22%, +1.14% |
| Drag | +12.63%, +8.20%, +9.80%, +12.11%, +17.51% | +4.89%, +0.17%, +1.81%, +2.13%, +8.26% | +5.13%, -0.29%, +2.25%, +1.99%, +7.69% |
| Scroll | +38.43%, +42.70%, +29.26%, +31.82%, +24.23% | +10.95%, +13.32%, +4.26%, +3.94%, +2.65% | +10.84%, +12.48%, +4.43%, +3.90%, +3.28% |

**Measured on a loaded machine.** Peak background CPU in the prepair,
start, and end samples is listed per pair. No compilation ran during
timing.

| Scene | Pair 1 | Pair 2 | Pair 3 | Pair 4 | Pair 5 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Typing | 8.87% | 16.89% | 8.19% | 9.22% | 19.26% |
| Hover | 23.68% | 17.34% | 16.08% | 13.94% | 11.11% |
| Drag | 17.42% | 12.18% | 12.77% | 13.38% | 12.48% |
| Scroll | 11.80% | 15.07% | 11.54% | 13.02% | 13.06% |

| Scene | Full-size damage-on GPU frame counts |
| --- | --- |
| Typing | 2,929 partial |
| Hover | 5 retained, 29,405 direct |
| Drag | 29,816 partial |
| Scroll | 30,776 direct |

## Recommendation

KEEP OPT-IN. INCONCLUSIVE: hover and scroll never executed the damage path (100% Direct). Typing/drag: GPU saving confirmed, CPU gate failed, cause unattributed.

The full-display hover timing counted five Retained priming frames and
29,405 Direct frames. The 100% Direct result applies after priming.
