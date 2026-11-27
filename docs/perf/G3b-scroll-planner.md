# G3b scroll declarations and Metal copy cost

Draft follow-up to [G3 scroll shift](G3-scroll-shift.md) on PR #191.

## Which baseline applies

G-R1 measured **0.587 ms** windowed at 2400×1600 and **0.890 ms** in a 3456×2234 borderless window. **1.713 ms** is the later G-R2e borderless result. G3 measured **0.6226 ms** windowed and **0.7834 ms** in native fullscreen at 3456×2168. All three use the same append-only row model and 120 Hz native scrolling in `apps/rustui-lab/src/g0.rs`. Scroll does not append during the scroll workload. The earlier reports do not establish one code change as the cause of the 1.713 ms figure. G-R1 used ScreenCaptureKit during timing, G-R2e and G3 did not. G3 uses a five-second warmup and Cocoa fullscreen, with 66 fewer drawable rows than borderless fullscreen. A same-build damage-off control alternated borderless, native, native, borderless for 60 seconds each. GPU means were 0.911, 0.777, 0.780 and 0.910 ms respectively. Both geometry pairs qualified for load, with no thermal limit. Window mode plus drawable size reproduced about 0.132 ms of difference. Neither geometry reproduced the G-R2e outlier. [G-R1 at 99bad7528](G-R1-hover-diagnosis.md) · [G-R2e at 75fa084d8](G-R2e-editor-damage.md) · [G3 at d6f9b9b10](G3-scroll-shift.md)

## Declaring scroll damage

`UniformList::scroll_damage_revision` lets the row owner identify content that translates rigidly in row coordinates. The Lab increments its revision when the append-only model length, inherited text style, text or background colour, or rem size changes. An unchanged revision lets the planner skip row signatures and compare the surrounding scene. A changed revision explicitly invalidates the list clip. The revision is a promise about visible pixels. A row with ink overhanging into its neighbours must invalidate the declaration if virtualization changes those pixels in the translated interior. The exposed strip and the opposite virtual-row boundary are repaired separately. The boundary can include glyphs from a row entering or leaving the set of painted rows.

The declaration path requires `RUSTUI_DAMAGE=1`, `RUSTUI_DAMAGE_SCROLL_SHIFT=1`, and `RUSTUI_DAMAGE_SCROLL_DECLARED=1`. Decorated lists, complex primitives, transformed glyphs, uploads, and intersecting foreground content keep the ordinary comparison. Verify mode computes full-scene signatures and the independent diff, projects the previous full reference through the actual copy rectangles, checks every copied pixel outside repairs, and compares the entire output with a fresh draw.

## GPU measurement

`RUSTUI_G3B_GPU=1` samples Metal encoder boundaries on shift commands. Blit timing covers every A→B copy, including stationary areas outside the clip. Render timing uses the union of the actual vertex and fragment intervals. The gap between these stages can include work from other passes. The final render pass samples retained B into the drawable. A repair extending across the exposed strip and additional content is counted as mixed. The analyzer checks expected pass counts, groups samples by command buffer, calibrates the timestamp frequency against Metal GPU time, and includes zero-cost categories. Copy percentage is the union of the blit interval and the final copy fragment interval divided by GPU command duration. Normal paired captures run with these samples disabled.

Windowed supplied 1,245 complete shift commands and fullscreen supplied 1,243. Every sampled shift command contained its expected blit, repair and presentation encoder readings. Values below are GPU microseconds per shift frame, mean / p95. Stage timings may overlap across encoders.

| GPU work | Windowed | Native fullscreen |
|---|---:|---:|
| A → B shift and stationary blits | 208.64 / 453.67 | 263.66 / 272.50 |
| Exposed-strip draw | 56.33 / 114.29 | 51.12 / 56.88 |
| Other damage draw | 67.89 / 106.37 | 64.22 / 71.50 |
| Single repair covering exposed and other damage | 0.38 / 0.00 | 0.38 / 0.00 |
| B → drawable render pass | 122.40 / 149.83 | 203.39 / 210.04 |
| B → drawable fragment stage that copies pixels | 89.43 / 101.75 | 158.75 / 163.79 |
| Both pixel copies, union of active intervals | 298.07 / 541.46 | 422.40 / 434.42 |
| Whole shift command | 543.01 / 1196.75 | 671.49 / 692.75 |

The two copies accounted for **54.89% windowed** and **62.91% fullscreen** of the sampled on-arm GPU command time. These were separate 12-second diagnostic runs.

## Removing the B→drawable copy

**L.** Present the completed retained image through an IOSurface-backed Core Animation layer. This needs a bounded pool of surfaces that Metal can render into, synchronization before Core Animation reads each surface, and a release signal before Metal recycles a surface. The layer must preserve alpha, colour space, backing scale, existing overlays and presentation ordering across resize and fullscreen. Measuring actual presentation time and power on this path is part of the experiment. A framebuffer-only Metal drawable cannot serve as the retained source for the following shift.

## Physical trackpad

Build `apps/latency-ref/g3_trackpad.swift` with `xcrun swiftc -O`. Pass an output CSV path to the executable. The first precise scroll event in the recorder window starts the 60-second interval. Each event records both point deltas, the window's backing scale, both device-pixel deltas, native gesture and momentum phase bits, and whether both axes are integral. Both recorder sessions timed out waiting for the first physical event. No 60-second gesture trace was obtained. Whole-pixel fractions for changed and momentum phases remain unmeasured, so the 80% snapping threshold is unevaluated.


## Paired scroll measurements

The Rust 1.98.0 release build used native Cocoa input at 120 events/s, G5 profile on, display sync on, a five-second still warmup and a 60-second measured window. Odd pairs ran off then on, even pairs on then off. Each pair received a thermal check and up to three retries. Background excludes the Lab, WindowServer and kernel_task. All thermal prechecks permitted timing. Both arms must be at most 100% of one core, with at most 15 percentage points between them. Process CPU comes from the Lab’s own user plus system time. GPU and CPU summaries use medians of qualified arm means. Planner means pool measured attempts. All verdicts use unrounded paired changes, with at least four qualified pairs.

### Windowed

| Pair (try) | Shift % off/on | GPU ms/frame off/on | Draw + present ms/attempt off/on | Process CPU s/s off/on | Plan µs off/on | Background % of one core off/on |
|---|---:|---:|---:|---:|---:|---:|
| 2 (1) | 0.00 / 99.98 | 0.6240 / 0.5519 | 0.7791 / 0.8840 | 0.2502 / 0.2596 | 8.02 / 26.10 | 95.7 / 96.4 |
| 3 (1) | 0.00 / 100.00 | 0.6278 / 0.5487 | 0.7658 / 0.9044 | 0.2471 / 0.2640 | 7.66 / 26.64 | 96.1 / 93.9 |
| 4 (2) | 0.00 / 100.00 | 0.6189 / 0.5419 | 0.7777 / 0.9239 | 0.2432 / 0.2590 | 7.73 / 25.94 | 92.8 / 90.3 |
| 5 (1) | 0.00 / 100.00 | 0.6210 / 0.5498 | 0.7656 / 0.9019 | 0.2440 / 0.2564 | 7.76 / 26.90 | 98.5 / 96.2 |

Qualified 4/5. GPU 0.6225 / 0.5493 ms/frame (GAIN), process CPU 0.2456 / 0.2593 s/s (REGRESSION), draw + present 0.7718 / 0.9031 ms/attempt, shift 0.000 / 99.996%, plan 7.79 / 26.39 µs (Δ +18.60 µs).

Paired GPU changes were -11.55%, -12.59%, -12.44%, -11.47%.

Paired process CPU changes were +3.77%, +6.82%, +6.49%, +5.10%.

Pair 1 exhausted its retries at background off/on 137.9/91.5%, 98.5/127.2%, 96.7/159.2% and 106.9/93.9%. Pair 4 qualified on retry after 103.8/94.3%.

### Fullscreen

| Pair (try) | Shift % off/on | GPU ms/frame off/on | Draw + present ms/attempt off/on | Process CPU s/s off/on | Plan µs off/on | Background % of one core off/on |
|---|---:|---:|---:|---:|---:|---:|
| 1 (1) | 0.00 / 100.00 | 0.7864 / 0.6767 | 0.9608 / 1.1157 | 0.3216 / 0.3373 | 7.87 / 28.68 | 50.6 / 56.5 |
| 2 (1) | 0.00 / 100.00 | 0.7731 / 0.6714 | 0.9553 / 1.1388 | 0.3239 / 0.3429 | 7.97 / 28.84 | 47.2 / 46.9 |
| 3 (1) | 0.00 / 100.00 | 0.7751 / 0.6678 | 0.9754 / 1.0930 | 0.3290 / 0.3315 | 8.07 / 27.54 | 49.2 / 47.3 |
| 4 (1) | 0.00 / 100.00 | 0.7742 / 0.6629 | 0.9268 / 1.1254 | 0.3197 / 0.3384 | 7.58 / 28.14 | 45.7 / 50.5 |
| 5 (1) | 0.00 / 100.00 | 0.7788 / 0.6733 | 0.9509 / 1.0891 | 0.3240 / 0.3309 | 7.69 / 27.76 | 47.9 / 45.2 |

Qualified 5/5. GPU 0.7751 / 0.6714 ms/frame (GAIN), process CPU 0.3239 / 0.3373 s/s (INCONCLUSIVE), draw + present 0.9553 / 1.1157 ms/attempt, shift 0.000 / 100.000%, plan 7.83 / 28.19 µs (Δ +20.36 µs).

Paired GPU changes were -13.94%, -13.16%, -13.85%, -14.38%, -13.55%.

Paired process CPU changes were +4.89%, +5.85%, +0.77%, +5.84%, +2.15%.

All pairs qualified on the first attempt.


Both geometries met the 40 µs planner target. The corresponding G3 planner deltas were 129.67 and 180.91 µs. Windowed fails the combined gate because every qualified pair increased process CPU by more than 3%. Fullscreen passes the specified gate with a GPU gain and an inconclusive process CPU result. Every fullscreen CPU change was positive.

## Correctness

| History | Compared frames | Expected decisions after prime | Mismatches |
|---|---:|---|---:|
| Slow down, slow up, momentum, sticky header, scrollbar, hover during scroll, bottom append, disjoint containers | 300 each | 300 Shift each | 0 |
| Fractional delta at 2× | 300 each | 300 Partial / fractional_delta | 0 |
| Oversized delta | 300 each | 300 Partial / delta_too_large | 0 |
| Nested overlapping containers | 300 each | 300 Direct / nested_overlap | 0 |
| Translucent overlay and sticky translucent overlay | 300 each | 300 Direct / overlay_translucent each | 0 |
| Stable declared rows with outside content changing | 300 each | 300 Shift / shift_declared | 0 |
| Declared content revision changes every third frame | 300 each | 200 declared shifts, 100 shifts repairing the changed clip | 0 |

The 15 histories compared **4,500 frames** with whole redraws after prime. A deliberately false stable revision triggered the independent verify check. Live Lab verify additionally checked 159 windowed and 85 native fullscreen shift outputs with zero mismatches. The nine existing native Metal proofs passed. Repository checks exited 0 with 1,875 passed, 7 ignored and 0 failed. Both Fast stacks passed exact reversal and replay, both owned-source checks passed, and candidate-mode source verification exited 0.
