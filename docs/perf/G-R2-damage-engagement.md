GATE: MISSED (best diff = 164.1 µs, after 2 attempts)

# G-R2d · signature collection on the repository compiler

Attempt 1 had the lowest measured typing difference and passed the hover and typing pixel oracles. Each diagnostic used one damage-off and one damage-on 60-second native typing run. The 60 µs decision uses the arithmetic mean of all in-window renderer planning records in each arm, including the existing damage CPU log write. The separately qualified workload pairs are below.

## Compiler

```toml
[toolchain]
channel = "1.98.0"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

Inside the worktree, plain `rustc --version` returned `rustc 1.97.1 (8bab26f4f 2026-07-14) (Homebrew)`. The repository helper returned `rustc 1.98.0 (88d9e12ae 2026-08-18)`. `scripts/check.sh` sources `scripts/pinned-toolchain.sh`, which reads the TOML channel and uses `rustup which` and `rustup run`. The check script also exports the 1.98.0 compiler and rustdoc explicitly and prepends their directory to PATH. Builds and checks used that helper. The toolchain files and the macOS `map_or_default` calls have an empty diff.

## Planner diagnosis

Values are mean / nearest-rank p95 in µs. The fixed-size recorder assigns each nanosecond to one sub-phase. Baseline is the fresh, pre-fix G-R2d build with the separate record-write timer.

| Planner sub-phase | Baseline off | Baseline on | Attempt 1 off | Attempt 1 on | Attempt 2 off | Attempt 2 on |
|---|---:|---:|---:|---:|---:|---:|
| `plan_us` | 127.1 / 226.2 | 311.8 / 536.0 | 113.0 / 237.5 | 277.1 / 441.5 | 105.8 / 247.0 | 287.1 / 449.3 |
| `preflight_us` | 1.3 / 2.3 | 1.8 / 3.3 | 1.1 / 2.5 | 1.9 / 3.5 | 1.6 / 3.0 | 2.4 / 4.1 |
| `atlas_and_backoff_us` | 0.0 / 0.0 | 1.5 / 2.8 | 0.0 / 0.0 | 1.6 / 3.0 | 0.0 / 0.0 | 1.4 / 2.9 |
| `prime_snapshot_us` | 0.0 / 0.0 | 0.0 / 0.0 | 0.0 / 0.0 | 0.0 / 0.0 | 0.0 / 0.0 | 0.0 / 0.0 |
| `signature_collection_us` | 0.0 / 0.0 | 67.4 / 164.0 | 0.0 / 0.0 | 65.5 / 90.1 | 0.0 / 0.0 | 49.5 / 72.6 |
| `fixed_compare_us` | 0.0 / 0.0 | 0.3 / 0.7 | 0.0 / 0.0 | 0.4 / 0.8 | 0.0 / 0.0 | 0.5 / 1.0 |
| `match_unchanged_us` | 0.0 / 0.0 | 69.1 / 98.7 | 0.0 / 0.0 | 70.8 / 100.0 | 0.0 / 0.0 | 64.7 / 96.4 |
| `scroll_vote_us` | 0.0 / 0.0 | 5.9 / 9.9 | 0.0 / 0.0 | 6.3 / 10.9 | 0.0 / 0.0 | 5.5 / 10.3 |
| `paint_order_us` | 0.0 / 0.0 | 17.9 / 26.4 | 0.0 / 0.0 | 14.3 / 22.1 | 0.0 / 0.0 | 16.0 / 23.9 |
| `shift_match_us` | 0.0 / 0.0 | 0.0 / 0.0 | 0.0 / 0.0 | 0.0 / 0.0 | 0.0 / 0.0 | 0.0 / 0.0 |
| `rect_merge_us` | 0.0 / 0.0 | 3.5 / 5.9 | 0.0 / 0.0 | 3.9 / 6.5 | 0.0 / 0.0 | 3.3 / 5.3 |
| `build_plan_us` | 0.0 / 0.0 | 2.1 / 3.5 | 0.0 / 0.0 | 2.2 / 3.6 | 0.0 / 0.0 | 1.1 / 2.0 |
| `select_mode_us` | 0.0 / 0.0 | 2.0 / 3.6 | 0.0 / 0.0 | 1.8 / 3.5 | 0.0 / 0.0 | 1.8 / 3.3 |
| `emit_cpu_record_us` | 125.8 / 224.0 | 140.2 / 237.2 | 111.9 / 234.8 | 108.4 / 225.9 | 104.2 / 244.9 | 141.1 / 231.5 |

| Diagnostic | Mean on − off | Renderer records off / on | Hashed / replayed per active typing frame |
|---|---:|---:|---:|
| Baseline | +184.7 µs | 583 / 587 | 888.5 / 1.0 |
| Attempt 1 | +164.1 µs | 584 / 582 | 888.5 / 1.0 |
| Attempt 2 | +181.3 µs | 583 / 581 | 888.5 / 1.0 |

Attempt 2 reduced matching by 6.2 µs. The synchronous CPU-record write contributed −3.6 µs to the attempt-1 on/off difference and +36.9 µs to attempt 2. Both writes were included in the gate as specified.

**Attempt 1, fixes 1 and 4.** `RUSTUI_G_R2_VIEW_SIGNATURES=1` records the existing generational GPUI view key. A slot-indexed Vec holds the previous and current per-view signature lists. Reuse requires the exact operation that GPUI replayed from a clean paint range. A repaint receives a fresh identity. Atlas freshness and current paint order are applied when reusing a signature. Rounded groups retain their child provenance and rebuild their aggregate. The collector recycles its output vectors across frames and clip depths.

The separate 60-second sizing capture yielded 1,749 nonempty view/frame observations. There were 583 counts of 1, 583 of 5, 288 of 883 and 295 of 884. Every observed signature had a view owner. Nearest-rank p95 was **884**, which is the per-view SmallVec inline capacity. Only one signature per typing frame qualified as exact clean replay on average. The editor repainted the view that owns nearly all glyphs.

**Attempt 2, fix 2.** Matching unchanged primitives was the largest positive contribution to the attempt-1 difference (70.8 µs). It pairs a multiset across the two entire scenes, including duplicate signatures and fresh atlas draws. The paint replay flag alone cannot establish that pairing. The trial matcher retained the previous-index chains, head map, matched indices and kept flags. It clears the buffers between calls. The two lists of changed indices each have SmallVec inline capacity 20, their measured typing p95 over 141 general-plan frames. For inputs of at most 16 old primitives, it finds each head by linear search in a SmallVec of key/value pairs. Larger inputs use the existing FxHashMap. Missing heads and exhausted chains must guard index accesses. There was no interchangeable numeric choice requiring `select_unpredictable` in this phase. NEON inspection did not apply. No crates were added. The existing `smallvec` and hash-map dependencies were used.

## Allocations and pixel correctness

The allocation capture used a separate instrumented release run per setting, without compiling during capture. The optional Lab dylib counts main-thread calls to malloc, calloc, realloc, aligned_alloc and posix_memalign. The signature scope covers signature collection, the renderer scope covers MetalRenderer::draw. These numbers exclude other threads and allocator entry points that bypass those functions. Means use all frames in the 15-second measured window. The allocation interposer was absent during timing.

| Damage / view cache | Renderer calls/frame mean / p95 | Signature calls/frame mean / p95 / max | Frames |
|---|---:|---:|---:|
| Off / off | 43.545 / 43 | 0 (collection bypassed) | 145 |
| On / off | 104.069 / 106 | 1.0000 / 1 / 1 | 145 |
| On / on | 103.097 / 105 | 0.0069 / 0 / 1 | 145 |

Of 145 opt-in collection scopes, 144 made zero measured heap calls. The remaining call occurred on a Copy frame about 1.31 s into the window. All 133 subsequent signature scopes in that run recorded zero. This establishes zero allocations in the observed steady sequence, with one exception over the full window.

| Native runtime byte oracle | Hover Partial frames | Typing Partial frames | Mismatches |
|---|---:|---:|---:|
| Attempt 1 | 838 | 568 | 0 |
| Attempt 2 | 847 | 566 | 0 |

The native proof also replays a clean clipped view, changes its content at the same location, requires a new signature, excludes Copy/Skip for the repaint, and checks the changed pixels against a whole draw. A companion test keeps the same numbered view slot across clean replay, repaint, and SlotMap generation replacement. It requires the edited signature on the following replay and rejects the retired owner’s cached entry. Both tests run under the existing nine native Metal proofs. The suite includes its deliberately injected damage mismatch, which must be detected.

To repeat the allocation comparison, compile the Lab-only helper with `xcrun clang -O2 -dynamiclib apps/latency-ref/g_r2_allocations.c -o target/headstart/g5-1/libGR2Allocations.dylib`. Set `RUSTUI_G_R2_ALLOCATIONS=1` only for the diagnostic run. For workload timing, use `RUSTUI_G_R2_VIEW_SIGNATURES=1 python3 scripts/run_g_r2.py matrix --output <new-directory>`. The sizing-only switch is `RUSTUI_G_R2_VIEW_SIZES=1`. Raw capture files are local measurement artifacts.

## Qualified workload pairs

Each 60-second arm used the same release build, native Cocoa input, 3456 × 2234 drawable, G5 profile, display sync on and buffered logging. The order alternated by pair. The candidate view-tagging switch was set in both arms. With damage off, the renderer bypassed signature collection. The load gate excludes the Lab, WindowServer and kernel_task. Both arms must be ≤100% of one core and differ by ≤15 percentage points. Each pair gets at most three retries. A CPU speed limit below 100 also consumes a retry. At least four qualified pairs are required.

Arm values are off → on. GPU ms/frame, draw+present ms/attempt and process CPU s/s are medians of qualified arm means. Partial percentages and mean plan µs use pooled measured records. Draw+present uses the 3% damage gate, scroll uses the 2% process CPU gate. PASS requires every qualified pair at or below its limit, FAIL every pair above it. The process CPU verdict uses the P rule across qualified pairs.

| Workload | Qualified | Partial % | GPU ms/frame | Draw+present ms/attempt | Plan µs | Process CPU s/s | Gate | Process CPU verdict |
|---|---:|---:|---:|---:|---:|---:|---|---|
| typing | 5/5 | 0.00 → 97.15 | 1.037 → 0.490 | 1.989 → 2.227 | 106.7 → 264.5 | 0.1032 → 0.1061 | FAIL | INCONCLUSIVE |
| hover | 5/5 | 0.00 → 99.82 | 1.310 → 0.970 | 2.089 → 2.128 | 88.2 → 154.6 | 0.3696 → 0.3744 | INCONCLUSIVE | INCONCLUSIVE |
| drag | 5/5 | 0.00 → 99.99 | 1.744 → 1.215 | 0.620 → 0.637 | 111.4 → 137.8 | 0.2415 → 0.2424 | INCONCLUSIVE | INCONCLUSIVE |
| scroll | 5/5 | 0.00 → 0.00 | 1.719 → 1.717 | 1.158 → 1.396 | 120.4 → 374.7 | 0.3504 → 0.3614 | INCONCLUSIVE | INCONCLUSIVE |

| Workload | Qualified paired draw+present changes | Qualified paired process CPU changes |
|---|---|---|
| typing | +15.63%, +14.42%, +11.04%, +11.41%, +6.55% | +2.82%, +3.21%, +1.86%, +3.47%, -0.25% |
| hover | +3.19%, +1.00%, -4.76%, +1.85%, +7.33% | +6.16%, +4.63%, -1.79%, +3.32%, +5.90% |
| drag | +1.89%, +2.94%, +3.19%, +8.70%, -0.39% | +0.05%, +0.38%, -0.78%, +5.12%, -1.48% |
| scroll | +23.18%, +21.49%, +18.18%, +19.42%, +16.30% | +4.81%, +4.22%, +2.12%, +2.87%, +0.57% |

All 20 requested pairs qualified in 26 attempts. Typing exceeded +3% draw+present CPU in every pair even though GPU time fell by 0.547 ms/frame. Scroll stayed Direct. Four scroll pairs exceeded the +2% process CPU limit and one was within it, giving an inconclusive scroll gate.

### Load and thermal state by pair attempt

Values are off / on, as percentages of one logical core. Thermal codes below identify the complete `pmset -g therm` status text for each attempt. No attempt reported a CPU speed limit or a thermal warning.

| Scene | Pair.try | Background | WindowServer | kernel_task | Thermal | Result |
|---|---:|---:|---:|---:|---|---|
| typing | 1.1 | 56.8 / 33.9 | 8.3 / 6.7 | 6.7 / 6.7 | T1 | load retry |
| typing | 1.2 | 37.4 / 59.4 | 6.7 / 6.7 | 6.7 / 8.3 | T1 | load retry |
| typing | 1.3 | 26.7 / 26.0 | 6.7 / 8.3 | 6.7 / 6.7 | T1 | qualified |
| typing | 2.1 | 86.6 / 31.0 | 6.7 / 6.7 | 11.7 / 6.7 | T1 | load retry |
| typing | 2.2 | 32.0 / 37.8 | 6.7 / 8.3 | 6.7 / 6.7 | T1 | qualified |
| typing | 3.1 | 28.3 / 24.6 | 6.7 / 8.3 | 6.7 / 6.7 | T1 | qualified |
| typing | 4.1 | 30.7 / 26.0 | 6.7 / 8.3 | 6.7 / 6.7 | T1 | qualified |
| typing | 5.1 | 59.7 / 27.9 | 6.7 / 6.7 | 8.3 / 6.7 | T1 | load retry |
| typing | 5.2 | 76.4 / 31.0 | 6.7 / 6.7 | 10.0 / 6.7 | T1 | load retry |
| typing | 5.3 | 28.2 / 29.5 | 6.7 / 6.7 | 6.7 / 6.7 | T1 | qualified |
| hover | 1.1 | 18.1 / 31.6 | 46.7 / 48.3 | 11.7 / 13.3 | T1 | qualified |
| hover | 2.1 | 28.7 / 22.6 | 46.7 / 46.7 | 11.7 / 11.7 | T1 | qualified |
| hover | 3.1 | 26.7 / 30.6 | 45.0 / 46.7 | 10.0 / 10.0 | T1 | qualified |
| hover | 4.1 | 28.2 / 26.8 | 45.0 / 46.7 | 10.0 / 11.7 | T1 | qualified |
| hover | 5.1 | 42.9 / 30.9 | 43.3 / 46.7 | 11.7 / 11.7 | T1 | qualified |
| drag | 1.1 | 30.7 / 31.1 | 48.3 / 48.3 | 11.7 / 11.7 | T1 | qualified |
| drag | 2.1 | 27.4 / 23.6 | 46.7 / 48.3 | 11.7 / 11.7 | T1 | qualified |
| drag | 3.1 | 29.6 / 39.3 | 45.0 / 48.3 | 11.7 / 11.7 | T1 | qualified |
| drag | 4.1 | 27.7 / 26.5 | 48.3 / 48.3 | 10.0 / 11.7 | T1 | qualified |
| drag | 5.1 | 21.1 / 24.1 | 46.7 / 48.3 | 11.7 / 11.7 | T1 | qualified |
| scroll | 1.1 | 46.4 / 38.6 | 40.0 / 41.7 | 13.3 / 13.3 | T1 | qualified |
| scroll | 2.1 | 38.7 / 56.4 | 40.0 / 41.7 | 13.3 / 13.3 | T1 | load retry |
| scroll | 2.2 | 36.9 / 44.4 | 40.0 / 40.0 | 11.7 / 13.3 | T1 | qualified |
| scroll | 3.1 | 42.3 / 36.7 | 41.7 / 40.0 | 13.3 / 13.3 | T1 | qualified |
| scroll | 4.1 | 49.2 / 37.8 | 41.7 / 40.0 | 13.3 / 13.3 | T1 | qualified |
| scroll | 5.1 | 35.4 / 38.6 | 40.0 / 40.0 | 13.3 / 13.3 | T1 | qualified |

T1 means macOS reported no recorded thermal warning, performance warning, or CPU power status. A numeric speed limit was unavailable.

## Verification

`scripts/check.sh` exited 0 with 1,875 passed, 7 ignored and 0 failed across 7 groups. All nine native Metal proofs passed. Both candidate Fast stacks passed full exact reverse/apply verification, and integration source verification exited 0. Attempt 2 was reversed before the workload pairs. All new switches default off.

---

# G-R2 damage engagement and renderer CPU

Damage remains opt-in. The attribution, early rejection and order
tracing switches also default to off.

## Method

Attribution uses `RUSTUI_G_R2_ATTRIBUTION=1`. The Metal renderer keeps
integer nanosecond counters on the stack and records each result in a
fixed-capacity array. It formats the records at exit. `plan_us`
includes signature creation and selection, `encode_us` includes
pre-acquisition instance preparation and frame encoding, and
`present_us` covers the rest of the Metal draw call, including
drawable acquisition, admission and submission. The reason is an enum.
Renderer durations are wall time on the render thread. Drawable waits
can contribute to those durations.

The runner reuses G-R1's 3456×2234 native Cocoa workload, G5 profile,
display sync, buffered logging and 60-second measured windows. It
samples background CPU with the second sample of a ten-second
`top` interval. Total machine CPU is converted to percentage of one
logical core after documented process exclusions. A pair starts only
below 5%. After the initial sample and up to three retries, the runner
records an ineligible pair as SKIPPED. Each sample and its top five
background processes are saved with that pair.

## Step 1: Typing and drag attribution

| Workload | Qualified pairs | Counter | Off mean / p95 (µs) | On mean / p95 (µs) |
| --- | ---: | --- | ---: | ---: |
| Typing | 0 / 5 | `plan_us` | — | — |
| Typing | 0 / 5 | `encode_us` | — | — |
| Typing | 0 / 5 | `present_us` | — | — |
| Drag | 0 / 5 | `plan_us` | — | — |
| Drag | 0 / 5 | `encode_us` | — | — |
| Drag | 0 / 5 | `present_us` | — | — |

All ten pairs were skipped. There is no qualified evidence to identify
a counter that explains at least 80% of the draw+present increase.
The Metal proof run emitted Retained and Skip samples with zero records
lost. Those samples exercise the recorder and are outside the timing
matrix.

## Step 2: Early rejection and scroll CPU gate

`RUSTUI_G_R2_EARLY_REJECT=1` rejects a shift after the planner detects
the move and before it builds the shift region, match pool, or dirty
rectangles. A separate check rejects an oversized single Metal redraw
box before rectangle coalescing or plan construction. The native
planner proofs cover both paths and assert that no damage plan was
built.

| Scroll CPU gate | Qualified pairs | Off CPU s/s | On CPU s/s | Result |
| --- | ---: | ---: | ---: | --- |
| On ≤ off + 2% | 0 / 5 | — | — | SKIPPED |

## Step 3: Hover order diagnosis

`RUSTUI_G_R2_ORDER_TRACE=1` records the first 20 order rejections,
including both element path hashes, the previous and current paint
indices, and the bounds and clipped reach. The minimal repro moves a
highlight between two rows and reproduces the 2→3 / 3→2 rank inversion.
The test covers both disjoint and intersecting reach. The live hover
probe was skipped under the same quiet rule. The observed G-R1 rank
values alone cannot establish whether the real primitives overlap, so
the existing order guard remains in force.

## Step 4: Metal scroll shift

In the G-R1 60-second scroll trace at 2400×1600, all 5,289 accepted
shift plans emitted the exact fallback code `shift_unimplemented`.
The planner had accepted the shift, and Metal's frame selector
classified any plan whose `shift` field was present as Direct. The
other 1,060 Direct frames used `area_rectangles`. The early-reject
candidate emits `shift_unsupported` at the first detected shift
candidate, before constructing a shift plan. It emits
`area_bounding_box` when a guaranteed over-limit final redraw box can
be identified before plan construction.

Estimated size **M**. Metal already has a persistent private BGRA8
texture and a render-copy pass to the drawable. An implementation
would blit the moving region from the retained texture to a scratch
texture, then from scratch back to the retained texture, avoiding an
overlapping self-copy. It would redraw the exposed and independently
dirty rectangles before the existing final render-copy. The drawable
could stay framebuffer-only because the blits use the retained and
scratch textures. The work includes scratch budgeting, ordering and
pixel-oracle coverage for scrolling, clipping, and atlas changes.

## Step 5: Background qualification

All 16 scheduled qualifications were SKIPPED. Across the 64 samples,
background load ranged from 40.5% to 223.6% of one logical core.
The requirement was strictly below 5.0%. An entry above 100% represents
more than one core's worth of background work.

| Workload | Pair or probe | Initial | Retry 1 | Retry 2 | Retry 3 | Result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Typing | 1 | 110.2% | 116.0% | 59.4% | 50.7% | SKIPPED |
| Typing | 2 | 55.7% | 95.1% | 59.5% | 49.0% | SKIPPED |
| Typing | 3 | 46.2% | 62.6% | 51.5% | 62.2% | SKIPPED |
| Typing | 4 | 40.5% | 56.8% | 54.9% | 65.8% | SKIPPED |
| Typing | 5 | 54.2% | 53.9% | 52.7% | 60.8% | SKIPPED |
| Drag | 1 | 47.3% | 83.0% | 61.1% | 46.9% | SKIPPED |
| Drag | 2 | 63.0% | 78.4% | 51.2% | 45.4% | SKIPPED |
| Drag | 3 | 50.5% | 67.0% | 50.9% | 54.6% | SKIPPED |
| Drag | 4 | 44.0% | 79.4% | 62.8% | 58.6% | SKIPPED |
| Drag | 5 | 61.6% | 79.1% | 55.6% | 58.3% | SKIPPED |
| Scroll | 1 | 57.4% | 222.5% | 72.6% | 56.1% | SKIPPED |
| Scroll | 2 | 53.3% | 76.8% | 70.7% | 60.6% | SKIPPED |
| Scroll | 3 | 49.3% | 66.6% | 52.2% | 52.3% | SKIPPED |
| Scroll | 4 | 91.0% | 223.6% | 174.8% | 62.5% | SKIPPED |
| Scroll | 5 | 60.1% | 54.6% | 57.0% | 62.0% | SKIPPED |
| Hover | 1 | 46.6% | 82.8% | 56.5% | 44.8% | SKIPPED |

## Verification

`scripts/check.sh` passed across seven groups with 1,875 passed,
7 ignored and 0 failed. Candidate source verification, exact replay
and reversal for both Fast stacks, and all nine native Metal proofs
passed. The release build used pinned Rust 1.98.0.

## G-R2b measurement protocol

This section supersedes the original 5%-of-one-core qualification.
Each arm samples Mach host CPU at the workload's measured boundaries
and subtracts the Lab's own `getrusage(RUSAGE_SELF)` user and system
CPU time over that same interval. The host's non-idle tick fraction
is multiplied by the online logical core count. The result is
expressed as percent of one logical core. Every other process counts
toward background load.

A pair qualifies when both arms are at or below 100% of one core and
the absolute arm-to-arm difference is at most 15 percentage points.
The runner alternates damage-off/on order by pair and allows up to
three retries after the initial attempt. At least four qualified pairs
are required for a verdict. The 60-second measured windows use the
same release build, G5 profile, native Cocoa input, display sync and
3456×2234 drawable. ScreenCaptureKit observation and its barcode are
disabled in timing arms because these measurements use Metal's GPU
timings and the Lab's own CPU counters. This removes the observer's
display-capture work from every timing arm. The standalone hover
probe keeps the observer for direct comparison with G-R1 and is
exempt from qualification.

The new order check uses a 1-device-pixel antialiasing margin and the
existing conservative effect reach (including Metal's 3×blur+1 shadow
reach). A reordered group is accepted only when its reversed pairs
have disjoint reach in both scenes. The fix is enabled for the hover
timing arm by `RUSTUI_G_R2_DISJOINT_ORDER=1` and defaults off.

### Live hover witnesses

The first 20 rejections had identical bounds for each element between
frames and disjoint inflated reach. Both paint groups reversed ranks
in every sample (2→3 and 3→2). The IDs below are GPUI element path
hashes. Bounds are device pixels, as (x, y) and width×height.

| Frame | Element A ID | A paint index old→new | A bounds | Element B ID | B paint index old→new | B bounds | Inflated overlap |
| ---: | --- | ---: | --- | --- | ---: | --- | --- |
| 0 | `e9c36dab68887ec7` | 41→41 | (83, 204) 19×22 | `9de6604bb42d50cf` | 22→21 | (83, 144) 19×22 | No |
| 1 | `ac5716fe8e3cbb41` | 60→60 | (83, 264) 19×22 | `e9c36dab68887ec7` | 41→40 | (83, 204) 19×22 | No |
| 2 | `23b71dcf5f3479b9` | 79→79 | (83, 324) 19×22 | `ac5716fe8e3cbb41` | 60→59 | (83, 264) 19×22 | No |
| 3 | `bb3f6db4d118a969` | 98→98 | (83, 384) 19×22 | `23b71dcf5f3479b9` | 79→78 | (83, 324) 19×22 | No |
| 4 | `4c8a8457f41905be` | 117→117 | (83, 444) 19×22 | `bb3f6db4d118a969` | 98→97 | (83, 384) 19×22 | No |
| 5 | `786e148f0cb1c150` | 136→136 | (83, 504) 19×22 | `4c8a8457f41905be` | 117→116 | (83, 444) 19×22 | No |
| 6 | `c790790b53601b46` | 155→155 | (83, 564) 19×22 | `786e148f0cb1c150` | 136→135 | (83, 504) 19×22 | No |
| 7 | `f3de8ba8cfe348f4` | 193→193 | (83, 684) 19×22 | `c790790b53601b46` | 155→154 | (83, 564) 19×22 | No |
| 8 | `83212b8b3537af2c` | 212→212 | (83, 744) 19×22 | `f3de8ba8cfe348f4` | 193→192 | (83, 684) 19×22 | No |
| 9 | `03926af850754f44` | 231→231 | (83, 804) 19×22 | `83212b8b3537af2c` | 212→211 | (83, 744) 19×22 | No |
| 10 | `074e7d8670b19344` | 250→250 | (83, 864) 19×22 | `03926af850754f44` | 231→230 | (83, 804) 19×22 | No |
| 11 | `f3c971313ce6d2e1` | 269→269 | (83, 924) 19×22 | `074e7d8670b19344` | 250→249 | (83, 864) 19×22 | No |
| 12 | `dde9d7a99f654282` | 288→288 | (83, 984) 19×22 | `f3c971313ce6d2e1` | 269→268 | (83, 924) 19×22 | No |
| 13 | `5e6c2bd5f8a1152f` | 307→307 | (83, 1044) 19×22 | `dde9d7a99f654282` | 288→287 | (83, 984) 19×22 | No |
| 14 | `75ef5689e03b9571` | 326→326 | (83, 1104) 19×22 | `5e6c2bd5f8a1152f` | 307→306 | (83, 1044) 19×22 | No |
| 15 | `d546b0434e4933a0` | 345→345 | (83, 1164) 19×22 | `75ef5689e03b9571` | 326→325 | (83, 1104) 19×22 | No |
| 16 | `8107065ac553867e` | 364→364 | (83, 1224) 19×22 | `d546b0434e4933a0` | 345→344 | (83, 1164) 19×22 | No |
| 17 | `48e0aa7c4684c3a2` | 383→383 | (83, 1284) 19×22 | `8107065ac553867e` | 364→363 | (83, 1224) 19×22 | No |
| 18 | `8107065ac553867e` | 363→364 | (83, 1224) 19×22 | `48e0aa7c4684c3a2` | 383→383 | (83, 1284) 19×22 | No |
| 19 | `d546b0434e4933a0` | 344→345 | (83, 1164) 19×22 | `8107065ac553867e` | 364→364 | (83, 1224) 19×22 | No |

The opt-in fix passed all 300 magenta-prefilled hover Partial frames
against a byte-for-byte full Metal redraw. Tests also cover an
overlapping reversal, AA-margin overlap, and shadow reach across
otherwise disjoint bounds.

### Top ten background processes before the first pair

Ten-second sample. CPU percentages use one logical core as the unit.

| Rank | Process | PID | CPU |
| ---: | --- | ---: | ---: |
| 1 | WindowServer | 624 | 68.6% |
| 2 | kernel_task | 0 | 21.8% |
| 3 | SkyComputerUseService | 63373 | 14.9% |
| 4 | bird | 1031 | 13.7% |
| 5 | Codex (Aperitif) | 63244 | 11.6% |
| 6 | Codex (Aperitif Renderer) | 63259 | 7.7% |
| 7 | cloudd | 972 | 4.2% |
| 8 | WidgetRenderer_Activities | 18412 | 4.2% |
| 9 | fileproviderd | 1032 | 3.5% |
| 10 | WhatsApp | 35693 | 3.0% |

### Pair qualification

The runner completed 43 pair attempts. **4** passed the
background limits. The two-hour cutoff stopped the remaining 16
pair slots before they exhausted all four attempts. The runner's
deadline was enforced during the wait, which continued after the
command connection timed out. No commands or polling ran alongside
the captures.

| Workload | Qualified pairs out of 5 |
| --- | ---: |
| Hover | 2 |
| Typing | 2 |
| Drag | 0 |
| Scroll | 0 |

The loads below are off / on, in percent of one core. The qualifying
attempt for a pair is marked ✓. TIMEBOX means retries remained.

| Workload | Pair | Load by attempt, off / on | Status |
| --- | ---: | --- | --- |
| Hover | 1 | 96.76% / 100.60% · 187.29% / 161.47% · 181.69% / 178.40% | TIMEBOX |
| Hover | 2 | 98.45% / 100.09% · 184.51% / 144.10% · 168.52% / 142.91% | TIMEBOX |
| Hover | 3 | 125.36% / 111.52% · 152.25% / 172.77% · 161.77% / 152.19% | TIMEBOX |
| Hover | 4 | 93.72% / 94.74% ✓ | QUALIFIED |
| Hover | 5 | 92.65% / 98.15% ✓ | QUALIFIED |
| Typing | 1 | 50.95% / 44.58% ✓ | QUALIFIED |
| Typing | 2 | 46.52% / 71.32% · 135.61% / 115.95% · 100.72% / 89.71% | TIMEBOX |
| Typing | 3 | 46.52% / 51.38% ✓ | QUALIFIED |
| Typing | 4 | 47.09% / 88.80% · 127.65% / 126.65% · 94.01% / 77.23% | TIMEBOX |
| Typing | 5 | 47.20% / 68.85% · 126.17% / 136.51% · 75.95% / 48.31% | TIMEBOX |
| Drag | 1 | 104.96% / 102.48% · 182.96% / 188.02% · 101.24% / 101.35% | TIMEBOX |
| Drag | 2 | 106.11% / 102.96% · 184.62% / 171.37% | TIMEBOX |
| Drag | 3 | 103.02% / 103.18% · 146.92% / 176.11% | TIMEBOX |
| Drag | 4 | 102.97% / 128.14% · 151.81% / 163.69% | TIMEBOX |
| Drag | 5 | 102.11% / 105.33% · 174.54% / 161.71% | TIMEBOX |
| Scroll | 1 | 173.66% / 183.30% · 164.13% / 158.97% | TIMEBOX |
| Scroll | 2 | 197.66% / 162.22% · 159.84% / 171.14% | TIMEBOX |
| Scroll | 3 | 189.52% / 172.88% · 156.92% / 157.03% | TIMEBOX |
| Scroll | 4 | 169.26% / 160.72% · 298.90% / 267.23% | TIMEBOX |
| Scroll | 5 | 185.95% / 172.88% · 268.01% / 232.89% | TIMEBOX |

### Hover engagement and cost

Damage count means executed Partial frames. Both qualified damage-on arms
presented 11,481 Partial frames out of 11,507 measured GPU-frame records.
The aggregate mode fraction and arm medians are below.

| Metric | Off | On |
| --- | ---: | ---: |
| Frames Damaged | 0.00% | 99.77% |
| GPU ms/frame | 1.041 | 0.681 |
| Draw+present CPU ms/attempt | 2.317 | 2.436 |
| Process CPU s/s | 0.3759 | 0.3831 |

**Verdict INCONCLUSIVE (2/5 qualified).** Draw+present paired changes
were +6.40% and +3.92%.
Process CPU changes were +2.74% and +1.13%.

### Typing and drag CPU attribution

Mean and nearest-rank p95 pool the renderer records inside the measured
windows of qualified pairs. The attribution share uses the average
within-pair counter delta divided by the same pairs' average
draw+present delta.

| Workload | Qualified pairs | Counter | Off mean / p95, µs | On mean / p95, µs |
| --- | ---: | --- | ---: | ---: |
| Typing | 2/5 | `plan_us` | 94.4 / 134.0 | 285.7 / 349.6 |
| Typing | 2/5 | `encode_us` | 180.8 / 258.1 | 199.4 / 271.6 |
| Typing | 2/5 | `present_us` | 102.3 / 143.0 | 104.3 / 132.9 |
| Drag | 0/5 | `plan_us` | — | — |
| Drag | 0/5 | `encode_us` | — | — |
| Drag | 0/5 | `present_us` | — | — |

No counter explains at least 80% of the 275.2 µs
typing increase in the two qualified pairs. Planning accounts for
69.5%. Typing attribution is provisional
and drag attribution remains unmeasured.

### Scroll early-rejection gate

**INCONCLUSIVE (0/5 qualified).** The +2% process CPU gate could not
be evaluated. The native unsupported-shift and oversized-box
proofs passed.

### G-R2b verification

`scripts/check.sh` passed with 1,875 passed, 7 ignored and 0 failed
across seven groups. Both Fast stacks, candidate source verification
and all nine native Metal proofs passed.

## G-R2c planner attribution

The runner samples host CPU and subtracts the Lab's own user and system
time and the CPU time of WindowServer and kernel_task over the arm.
Both excluded services remain visible in the results. The service
sampler uses targeted macOS `top` cumulative CPU readings at the
boundaries. When a service has accrued an hour of CPU time, `top`
reports whole seconds. The resulting boundary precision is about
1.7 percentage points of one core for a 60-second arm per service.

Before each pair the runner reads `pmset -g therm`. A CPU speed limit
below 100 consumes an attempt and waits 60 seconds before the retry.
Both arms must be at or below one core of background CPU, with a gap
at or below 15 percentage points. Four qualified pairs are required.
The runner proceeds in typing, hover, drag, then scroll order.

The table records the unqualified 60-second typing diagnostic completed
before the 1.97.1 steer. Values are mean / p95 in µs. The legacy
damage CPU record write was attributed to Preflight in the off arm and
Select mode in the on arm. The final probe records that write in its
own bucket. The stable build failed before that version could be measured.

| Planner phase | Before off | Before on | After off / on |
| --- | ---: | ---: | --- |
| `plan_us` | 98.2 / 140.4 | 271.1 / 359.0 | — |
| `preflight_us` | 98.2 / 140.4 | 1.9 / 2.5 | — |
| `atlas_and_backoff_us` | 0.0 / 0.0 | 1.3 / 1.8 | — |
| `prime_snapshot_us` | 0.0 / 0.0 | 0.0 / 0.0 | — |
| `signature_collection_us` | 0.0 / 0.0 | 70.6 / 100.8 | — |
| `fixed_compare_us` | 0.0 / 0.0 | 0.4 / 0.5 | — |
| `match_unchanged_us` | 0.0 / 0.0 | 64.8 / 90.0 | — |
| `scroll_vote_us` | 0.0 / 0.0 | 4.7 / 6.2 | — |
| `paint_order_us` | 0.0 / 0.0 | 14.0 / 19.5 | — |
| `shift_match_us` | 0.0 / 0.0 | 0.0 / 0.0 | — |
| `rect_merge_us` | 0.0 / 0.0 | 2.9 / 3.8 | — |
| `build_plan_us` | 0.0 / 0.0 | 2.0 / 2.8 | — |
| `select_mode_us` | 0.0 / 0.0 | 108.7 / 137.6 | — |
| `emit_cpu_record_us` | — | — | — |

Among the independently measured planner phases, signature collection
was largest at 70.6 µs mean. The existing replay check reused only
one signature per measured frame on average. The planner optimization
was never applied, so the 60 µs gate has only the pre-steer baseline.
Neither fix attempt was consumed. NEON is inapplicable. No crates were added.

| Diagnostic arm | Background, % core | WindowServer, % core | kernel_task, % core | Thermal gate |
| --- | ---: | ---: | ---: | --- |
| Off | 52.88% | 13.31% | 8.32% | Not applied to diagnostic |
| On | 30.32% | 13.31% | 8.32% | Not applied to diagnostic |

### Stable compiler blocker

The workspace declares Rust 1.98.0. The requested stable 1.97.1
release build was attempted with the existing lockfile and a local
toolchain override. Even with Cargo's version declaration check
bypassed for this build, the compiler produced three E0658 errors
in unchanged GPUI Fast macOS code. The calls to
`Option::map_or_default` are in
`crates/rustui-gpui-fast-macos/src/window.rs` at lines 942, 1017, and 1556.

Step 3 has 0/5 qualified pairs for typing, hover, drag and scroll.
All performance gates are unevaluated. There are no measured thermal
states for Step 3 because the first pair did not start. The
post-fix pixel oracles, `check.sh`, and the nine native Metal proofs
were not run on 1.97.1 after the build blocker.
