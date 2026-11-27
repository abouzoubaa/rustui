GATE TYPING: PASS (diff = 49.23 µs, after 2 attempts)
GATE SCROLL: PASS (diff = 0.76 µs)

# G-R2e · Editor-declared Metal damage

Both gates use pooled measured-window attempts from qualified pairs at 3456 × 2234. The typing limit is 60 µs and the scroll limit is 10 µs. Damage and editor declarations each require explicit opt-in.

Typing used Partial in every measured damage-on frame and reduced GPU time by 13%. Draw and present still exceeded the historical 3% CPU limit in all five typing pairs. Hover saved 29% GPU time per frame, with an inconclusive process CPU verdict. Scroll stayed Direct. Only the planner cost passed the scroll gate consistently; one of the five pairs exceeded the separate 2% process CPU limit.

The timebox ended with one qualified drag pair. Two attempts at pair 2 failed background qualification, and pairs 3–5 did not run. The drag row is descriptive only.

## Measurement overhead and planner phases

Before editor changes, a 60-second typing pair put every frame log after the end marker in both capture files. Both arms reported zero dropped records. The damage recorder uses a 32 MiB preallocated byte ring. The core recorder holds up to 524,288 timestamp records. The initial buffer held 131,072. A hover control capture exceeded it by 65,453 records and was discarded. Timing pairs in the final table use the resized buffer. The capture runner rejects overflow and premature frame-log writes. The difference between the old and new record-emission times includes run-to-run variation. The new emission times still include formatting and copying into memory.

Values are mean / nearest-rank p95, in µs per attempted present. These three diagnostic pairs are separate from the qualified workload matrix.

| Phase | Buffered baseline off | Buffered baseline on | Attempt 1 off | Attempt 1 on | Attempt 2 off | Attempt 2 on |
| --- | --- | --- | --- | --- | --- | --- |
| plan_us | 7.81 / 14.79 | 156.64 / 225.88 | 7.81 / 13.92 | 116.16 / 167.17 | 7.13 / 13.46 | 54.95 / 81.17 |
| preflight_us | 1.75 / 2.96 | 2.25 / 3.79 | 1.25 / 2.71 | 2.08 / 3.38 | 1.36 / 2.58 | 1.88 / 3.33 |
| atlas_and_backoff_us | 0.00 / 0.00 | 1.70 / 2.92 | 0.00 / 0.00 | 1.70 / 2.83 | 0.00 / 0.00 | 1.42 / 3.04 |
| prime_snapshot_us | 0.00 / 0.00 | 0.00 / 0.00 | 0.00 / 0.00 | 0.00 / 0.00 | 0.00 / 0.00 | 0.00 / 0.00 |
| signature_collection_us | 0.00 / 0.00 | 49.18 / 73.71 | 0.00 / 0.00 | 37.53 / 52.46 | 0.00 / 0.00 | 35.33 / 51.08 |
| fixed_compare_us | 0.00 / 0.00 | 0.42 / 0.83 | 0.00 / 0.00 | 0.46 / 0.79 | 0.00 / 0.00 | 8.72 / 12.62 |
| match_unchanged_us | 0.00 / 0.00 | 68.10 / 98.96 | 0.00 / 0.00 | 45.15 / 64.79 | 0.00 / 0.00 | 0.00 / 0.00 |
| scroll_vote_us | 0.00 / 0.00 | 4.98 / 8.79 | 0.00 / 0.00 | 2.65 / 4.96 | 0.00 / 0.00 | 0.00 / 0.00 |
| paint_order_us | 0.00 / 0.00 | 16.91 / 25.00 | 0.00 / 0.00 | 14.63 / 21.00 | 0.00 / 0.00 | 0.00 / 0.00 |
| shift_match_us | 0.00 / 0.00 | 0.00 / 0.00 | 0.00 / 0.00 | 0.00 / 0.00 | 0.00 / 0.00 | 0.00 / 0.00 |
| rect_merge_us | 0.00 / 0.00 | 3.42 / 6.67 | 0.00 / 0.00 | 2.25 / 4.21 | 0.00 / 0.00 | 0.00 / 0.00 |
| build_plan_us | 0.00 / 0.00 | 2.20 / 3.83 | 0.00 / 0.00 | 1.43 / 2.71 | 0.00 / 0.00 | 0.00 / 0.00 |
| select_mode_us | 0.00 / 0.00 | 1.53 / 2.96 | 0.00 / 0.00 | 1.67 / 3.08 | 0.00 / 0.00 | 1.56 / 3.00 |
| emit_cpu_record_us | 6.06 / 11.92 | 5.95 / 10.67 | 6.56 / 11.79 | 6.60 / 12.92 | 5.77 / 11.17 | 6.06 / 11.42 |

Before buffering, the G-R2d diagnostic recorded mean log emission of 111.9 µs off and 108.4 µs on. The buffered diagnostic measured about 6 µs in each arm. Both arms emitted every CPU and timing record after capture, with zero lost records.

Typing mean on minus off was 148.83 µs after buffering, 108.34 µs using editor declarations with the general matcher, and 47.83 µs in the second diagnostic pair, before the recorder capacity increase. Attempt 2 compares ordered streams by primitive kind and falls back to the general planner when the changed span would require a large redraw.

## Scroll and editor correctness

Interactive scroll containers compare the final clamped offset with the last observed offset, including scroll-to-item changes. Metal declines a changed offset at planner entry when early rejection is enabled. A native proof checks that the first observation and an unchanged offset are clean, then changes the offset and asserts Direct with zero calls to signature collection.

The editor identifies a shaped row by its immutable layout revision, placement, alignment, clip and inherited opacity. It declares the row footprint and paints it normally. The renderer groups eligible primitives at the same draw order. It handles glyph overhangs and undeclared paint through the regular signature path. Old and new caret and selection footprints participate in the diff. A line-count or wrap change also contributes a rectangle from the affected row to the bottom of the visible editor.

With `RUSTUI_DAMAGE=1 RUSTUI_DAMAGE_DECLARED=1 RUSTUI_DAMAGE_VERIFY=1`, the renderer also computes the ordinary full signature diff and reads back a full reference. It compares consecutive full references against the separate allowed regions, as well as comparing the presented result with the current full reference. The negative Metal test changes a middle region while declaring only two outer regions. It detects pixel (20, 58) even though that pixel lies inside the combined redraw box.

| Actual Kit editor operation | Verified Partial frames |
| --- | --- |
| Insert / backspace | 300 / 300 |
| Newline / remove newline | 300 / 300 |
| Multiline paste / undo | 300 / 300 |
| Extend / collapse selection | 300 / 300 |
| Blink off / on | 300 / 300 |

The fixture uses Kit’s editor, native font shaping, its Metal glyph atlas, a private test clipboard and a deterministic blink timer. It checks final text and selection, a visible change on each step, the selected Partial mode, and byte equality after magenta pre-fill.

## View cache memory and allocations

A separate measured window contained 435 view observations. Median signatures per view was 5. Inline capacity changed from 884 to 5. Both alternating lists keep their spill allocation when cleared. Bytes count the view cache structure, slot array, live entries, touched-slot vectors and spilled list capacities. Allocator bookkeeping and GPU textures are outside that count.

| Arm | Build | Median view-cache bytes | Signature allocations/frame | All renderer allocations/frame |
| --- | --- | --- | --- | --- |
| off | 884 inline | 0 | 0.000 | 43.56 |
| off | 5 inline | 0 | 0.000 | 43.56 |
| signatures | 884 inline | 382288 | 0.000 | 104.56 |
| signatures | 5 inline | 150016 | 0.007 | 103.60 |
| declared | 884 inline | 382288 | 0.000 | 84.75 |
| declared | 5 inline | 39424 | 0.007 | 52.58 |

Allocation counts use the interposed allocator in separate 15-second typing captures after warmup. The damage-off arm executes zero signature collections.

## Qualified workload pairs

Each pair used one frozen release build, direct native Cocoa input, the G5 profile, display sync on, buffered logging and 60 seconds per arm. Odd-numbered pairs ran off first. Each background load excludes the Lab, WindowServer and kernel_task. Both arms had to stay at or below one core, with a difference of at most 15 percentage points. The runner retried up to three times and checked thermal limits before each attempt. At least four pairs qualify a verdict.

Arm values are off → on. GPU, draw+present and process CPU rates are medians of qualified arm means. Percent Partial and mean plan time pool measured attempts. The historical draw+present gate uses 3% per pair, with the 2% process-CPU gate for scroll. The P verdict uses process CPU seconds per second.

| Workload | Qualified | Partial % | GPU ms/frame | Draw+present ms/attempt | Mean plan µs | Process CPU s/s | Workload gate | P verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing | 5/5 | 0.00 → 100.00 | 1.032 → 0.898 | 1.854 → 2.047 | 7.03 → 56.26 | 0.1004 → 0.1020 | FAIL | INCONCLUSIVE |
| scroll | 5/5 | 0.00 → 0.00 | 1.713 → 1.711 | 1.070 → 1.130 | 9.12 → 9.89 | 0.3435 → 0.3494 | INCONCLUSIVE | NEUTRAL |
| hover | 5/5 | 0.00 → 99.82 | 1.324 → 0.935 | 1.979 → 2.045 | 6.02 → 81.85 | 0.3544 → 0.3729 | INCONCLUSIVE | INCONCLUSIVE |
| drag | 1/5 | 0.00 → 100.00 | 1.785 → 1.207 | 0.549 → 0.565 | 8.96 → 37.51 | 0.2533 → 0.2533 | INCONCLUSIVE | INCONCLUSIVE |

| Workload | Paired process CPU changes, % |
| --- | --- |
| typing | +3.56, +0.17, +2.03, +2.39, +0.18 |
| scroll | +0.28, +1.87, +2.28, +0.91, +0.84 |
| hover | +6.98, +5.45, +0.95, +6.71, +5.21 |
| drag | +0.01 |

### Background and thermal qualification

Load values are percent of one core, off / on. Thermal entries come from `pmset -g therm`.

| Workload | Pair | Attempt | Background | WindowServer | kernel_task | Thermal state | Disposition |
| --- | --- | --- | --- | --- | --- | --- | --- |
| typing | 1 | 1 | 73.9 / 40.3 | 10.0 / 10.0 | 8.3 / 6.7 | No thermal warning or CPU limit reported | load retry |
| typing | 1 | 2 | 32.5 / 28.3 | 10.0 / 10.0 | 6.7 / 6.7 | No thermal warning or CPU limit reported | qualified |
| typing | 2 | 1 | 27.8 / 32.6 | 10.0 / 8.3 | 6.7 / 6.7 | No thermal warning or CPU limit reported | qualified |
| typing | 3 | 1 | 27.5 / 48.9 | 8.3 / 10.0 | 6.7 / 6.7 | No thermal warning or CPU limit reported | load retry |
| typing | 3 | 2 | 26.7 / 30.1 | 10.0 / 10.0 | 6.7 / 6.7 | No thermal warning or CPU limit reported | qualified |
| typing | 4 | 1 | 29.6 / 29.2 | 8.3 / 10.0 | 5.0 / 8.3 | No thermal warning or CPU limit reported | qualified |
| typing | 5 | 1 | 18.6 / 32.3 | 10.0 / 8.3 | 6.7 / 6.7 | No thermal warning or CPU limit reported | qualified |
| scroll | 1 | 1 | 60.5 / 65.8 | 40.0 / 40.0 | 13.3 / 15.0 | No thermal warning or CPU limit reported | qualified |
| scroll | 2 | 1 | 49.3 / 44.8 | 40.0 / 40.0 | 13.3 / 13.3 | No thermal warning or CPU limit reported | qualified |
| scroll | 3 | 1 | 51.7 / 40.7 | 40.0 / 41.7 | 13.3 / 13.3 | No thermal warning or CPU limit reported | qualified |
| scroll | 4 | 1 | 58.5 / 44.0 | 41.7 / 41.7 | 15.0 / 13.3 | No thermal warning or CPU limit reported | qualified |
| scroll | 5 | 1 | 37.9 / 38.1 | 41.7 / 41.7 | 13.3 / 13.3 | No thermal warning or CPU limit reported | qualified |
| hover | 1 | 1 | 22.2 / 25.8 | 46.7 / 46.7 | 11.7 / 11.7 | No thermal warning or CPU limit reported | qualified |
| hover | 2 | 1 | 17.6 / 26.7 | 46.7 / 48.3 | 11.7 / 13.3 | No thermal warning or CPU limit reported | qualified |
| hover | 3 | 1 | 53.0 / 25.0 | 45.0 / 48.3 | 11.7 / 11.7 | No thermal warning or CPU limit reported | load retry |
| hover | 3 | 2 | 27.3 / 35.2 | 45.0 / 46.7 | 11.7 / 11.7 | No thermal warning or CPU limit reported | qualified |
| hover | 4 | 1 | 34.4 / 25.4 | 45.0 / 46.7 | 11.7 / 11.7 | No thermal warning or CPU limit reported | qualified |
| hover | 5 | 1 | 28.1 / 42.4 | 45.0 / 46.7 | 11.7 / 11.7 | No thermal warning or CPU limit reported | qualified |
| drag | 1 | 1 | 23.2 / 26.1 | 48.3 / 48.3 | 11.7 / 11.7 | No thermal warning or CPU limit reported | qualified |
| drag | 2 | 1 | 137.9 / 26.7 | 45.0 / 48.3 | 15.0 / 11.7 | No thermal warning or CPU limit reported | load retry |
| drag | 2 | 2 | 103.9 / 103.8 | 45.0 / 48.3 | 11.7 / 13.3 | No thermal warning or CPU limit reported | load retry |

## Verification and reproduction

`scripts/check.sh` counted 1,875 passed, 7 ignored and 0 failed across 7 test groups.

All nine native Metal proofs, the editor oracle, and candidate-mode source verification exited 0. Both regenerated Fast stacks verified exact reverse and forward restoration across 111 ordered patches and one alternative.

Use the pinned 1.98.0 toolchain. From the repository root, the measurement and editor proof commands are

```sh
bash scripts/build_g5_refs.sh
RUSTUI_DAMAGE_DECLARED=1 RUSTUI_G_R2_VIEW_SIGNATURES=1 python3 scripts/run_g_r2.py matrix --output target/headstart/g-r2e-reproduction
python3 scripts/test_g_r2e_editor_oracle.py 300
```
