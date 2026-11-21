# G0 baseline — GPUI Fast / Metal

Measured 2026-11-20 on an Apple M1 Max (32 GPU cores, 64 GB RAM), macOS 27.0 (26A5425a), Rust 1.98.0 release profile. The built-in Retina display reported 120 Hz. Each scene used a 1200 × 800 point window at scale 2, dark appearance, `RUSTUI_DAMAGE=0`, `RUSTUI_FRAME_LOG=1` and `RUSTUI_FRAME_PROFILE=1`. Compilation and validation finished before capture; no screen capture or power sampler ran during these timings.

Each scene warms for two seconds, then measures 60 seconds. These are one-run native baselines, not repeated-run confidence intervals or battery measurements. In that initial Batch 1 capture every GPU record asserted mode `full`; partial redraw had not yet been implemented.

| Scene | Measured s | GPU samples | GPU mean / p95 ms per present | GPU ms per second | CPU mean ms per present | Power |
|---|---:|---:|---:|---:|---:|---|
| Idle caret | 60.157 | 114 | 0.4236 / 0.9565 | 0.8027 | 2.3762 | owner run pending |
| 10,000-line editor | 60.032 | 515 | 0.8792 / 1.3230 | 7.5422 | 1.5051 | owner run pending |
| List row hover | 60.008 | 7193 | 0.6395 / 0.9676 | 76.6571 | 2.6852 | owner run pending |
| Auto-scrolling log | 60.005 | 575 | 0.6082 / 0.8117 | 5.8281 | 7.2872 | owner run pending |
| Card drag | 60.001 | 7195 | 0.7998 / 1.2256 | 95.9109 | 1.2997 | owner run pending |

GPU time is `GPUEndTime - GPUStartTime` from completed Metal command buffers. CPU time per present pairs the latest `draw` with `present`; a cached present contributes zero draw time. GPU completion and CPU submission timestamps can put one boundary frame in different sample windows. Input events may coalesce into fewer presents.

CPU phase means below are milliseconds per recorded phase invocation. Phases nest inside draw; do not add these columns to estimate a frame. Layout can run more than once in one draw.

| Scene | Setup | Build | Layout | Prepaint | Paint | Draw | Present |
|---|---:|---:|---:|---:|---:|---:|---:|
| Idle caret | 0.0057 | 0.2918 | 0.0057 | 0.4594 | 0.0816 | 2.1903 | 0.1859 |
| 10,000-line editor | 0.0021 | 0.1610 | 0.0012 | 0.7278 | 0.1059 | 1.3895 | 0.1020 |
| List row hover | 0.0017 | 0.6444 | 0.0004 | 2.1676 | 0.1072 | 3.3050 | 0.0615 |
| Auto-scrolling log | 0.0050 | 0.4991 | 0.0034 | 5.2811 | 0.4031 | 7.2040 | 0.0832 |
| Card drag | 0.0029 | 0.0949 | 0.0379 | 0.3131 | 0.0512 | 1.4584 | 0.1375 |

Workloads use normal GPUI input and notifications, with no forced redraw loop:

- Idle: focused RustUI TextField with its normal blinking caret; no scripted input during measurement.
- Typing: RustUI CodeEditor containing 10,000 Rust source lines; 10 keystrokes/s alternating insertion and backspace, 600 keystrokes completed.
- Hover: a 5,000-row RustUI List, 30-point rows; pointer sweep at 120 events/s, 7,200 events completed.
- Log: 1,000 initial records, 24-point rows; 600 records appended at 10 lines/s with the viewport following the newest line.
- Drag: a 200 × 120 point card moved across a board and another card, at 120 events/s; 7,200 drag moves and the final drop completed.

Run `RUSTUI_DAMAGE=0 ./scripts/run_g0_bench.sh` or `RUSTUI_DAMAGE=1 ./scripts/run_g0_bench.sh` from the repository root (default: `0`). It builds the Lab, runs all five scenes sequentially and writes frame logs plus `summary.json` into a new directory under `target/headstart/performance/`. It rejects missing or invalid completion markers, short captures, missing CPU/GPU samples and unexpected GPU modes. Lab without `--g0-scene` still launches the canonical Playground.

The owner runs `./scripts/run_g0_power.sh` from the repository root. It authenticates before measurement, then captures each scene with `sudo powermetrics --samplers gpu_power,cpu_power -i 1000 -n 60`. It saves raw per-scene power samples, the same frame summaries, and CPU/GPU means in `power-summary.json`. Power remains **owner run pending**; no sudo command was run for this baseline. Owner visual review also remains pending.

## Batch 2 — damage off and on

Measured 2026-11-21 with the same host, Rust 1.98.0 release profile, 1200 × 800 point window, scale 2 and dark appearance. Each of the five scenes completed 60 seconds after its warmup, first with `RUSTUI_DAMAGE=0`, then with `RUSTUI_DAMAGE=1`. Compilation and validation finished before capture. These are one-run comparisons; power remains **owner run pending**.

GPU values below are mean / p95 milliseconds per submitted frame; Skip has no GPU submission or GPU time. CPU is draw + present milliseconds per attempted present, including attempts that Skip. GPU milliseconds per second captures both submission count and GPU duration. Historical mode `full` is called `direct` by the current recorder.

| Scene | GPU off mean / p95 ms | GPU on mean / p95 ms | CPU off ms/attempt | CPU on ms/attempt | Power |
|---|---:|---:|---:|---:|---|
| Idle caret | 0.3061 / 0.5568 | 0.2200 / 0.5056 | 2.5291 | 2.2323 | owner run pending |
| 10,000-line editor | 0.6324 / 1.0719 | 0.2998 / 0.5954 | 1.5554 | 1.8074 | owner run pending |
| List row hover | 0.3637 / 0.6022 | 0.3812 / 0.6193 | 2.4550 | 2.7296 | owner run pending |
| Auto-scrolling log | 0.4494 / 0.7469 | 0.4641 / 0.7528 | 6.8261 | 7.5787 | owner run pending |
| Card drag | 0.6069 / 0.9864 | 0.3187 / 0.6220 | 1.3187 | 1.4433 | owner run pending |

| Scene | GPU ms/s off | GPU ms/s on | Prepaint off ms | Prepaint on ms |
|---|---:|---:|---:|---:|
| Idle caret | 0.5809 | 0.4170 | 0.4641 | 0.4094 |
| 10,000-line editor | 5.4745 | 2.1822 | 0.7718 | 0.7514 |
| List row hover | 43.5984 | 45.6628 | 2.1426 | 2.2898 |
| Auto-scrolling log | 4.3062 | 4.4468 | 4.8004 | 5.0439 |
| Card drag | 72.7255 | 38.2010 | 0.3147 | 0.3199 |

Frame-mode counts are from the measurement interval, excluding warmup. GPU completion timestamps can differ from CPU submission timestamps at the interval boundary. Retained means a whole draw into the kept texture followed by its copy; Direct draws whole into the drawable.

| Scene | Damage | Skip | Copy | Partial | Retained | Direct |
|---|---:|---:|---:|---:|---:|---:|
| Idle caret | 0 | 0 | 0 | 0 | 0 | 114 |
| Idle caret | 1 | 0 | 0 | 114 | 0 | 0 |
| 10,000-line editor | 0 | 0 | 0 | 0 | 0 | 520 |
| 10,000-line editor | 1 | 81 | 0 | 437 | 0 | 0 |
| List row hover | 0 | 0 | 0 | 0 | 0 | 7194 |
| List row hover | 1 | 0 | 212 | 0 | 3288 | 3688 |
| Auto-scrolling log | 0 | 0 | 0 | 0 | 0 | 575 |
| Auto-scrolling log | 1 | 0 | 0 | 0 | 0 | 575 |
| Card drag | 0 | 0 | 0 | 0 | 0 | 7190 |
| Card drag | 1 | 1 | 1260 | 5933 | 0 | 0 |

Findings:

- GPU milliseconds per second fell for idle, typing and drag. Native list hover recorded no Partial frames; Retained and Direct dominated that scene. The auto-scrolling log remained entirely Direct. These fallbacks were measured, not optimized in this batch.
- CPU time per attempted present rose 16.2% for typing, 11.2% for hover, 11.0% for the log and 9.5% for drag. This G0 comparison does not establish the separate F0 CPU acceptance gate or the G2-6 default-on decision. All damage switches remain off by default.
- The auto-scrolling log scene spends about **5.3 ms in CPU prepaint per frame** in the initial G0 baseline (5.2811 ms). The new off/on runs measured 4.8004 / 5.0439 ms per recorded prepaint invocation. This CPU finding is recorded without a fix; the Lab scene is unchanged.

Reproduce both captures from the repository root:

```sh
RUSTUI_DAMAGE=0 ./scripts/run_g0_bench.sh
RUSTUI_DAMAGE=1 ./scripts/run_g0_bench.sh
```

## Batch 3 — G2-7 CPU budget

Measured 2026-11-21 with the same host, release toolchain, window size, scale and appearance. Two complete off/on pairs ran, each scene measuring 60 seconds after warmup. The second pair includes the fixed-position comparison shortcut and corrected whole-frame fallback hysteresis. Builds and checks finished before each pair. These are individual native captures, not confidence intervals. Power remains **owner run pending**.

GPUI Fast 0.1.14 has no renderer-facing per-primitive replay provenance after scene sorting and rounded-clip compilation. The candidate adds operation identity in a new core `src/fast/damage/replay.rs`, with small Scene and clip-compiler hooks. Exact replay reuses cached signatures; current draw order and atlas freshness are updated. Newly painted primitives still hash. Known unsupported scenes, geometry changes, lost atlas contents, synchronous presents and disabled verification windows are rejected before hashing. A whole retained baseline defers its signature work until a later frame is actually comparable.

The first pair measured the following CPU changes before the second optimization. All five scenes completed in both pairs; no scene was omitted from the gate.

| Scene | First off ms/attempt | First on ms/attempt | Change |
|---|---:|---:|---:|
| Idle caret | 2.4236 | 2.5871 | +6.75% |
| 10,000-line editor | 1.5889 | 1.6632 | +4.68% |
| List row hover | 2.4310 | 2.5604 | +5.32% |
| Auto-scrolling log | 7.4092 | 7.3221 | -1.18% |
| Card drag | 1.4176 | 1.6198 | +14.26% |

Final candidate comparison:

| Scene | GPU off ms/present | GPU on ms/present | GPU off ms/s | GPU on ms/s | CPU off ms/attempt | CPU on ms/attempt | CPU change | Power |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| Idle caret | 0.3233 | 0.2212 | 0.6125 | 0.4195 | 2.5655 | 2.7892 | +8.72% | owner run pending |
| 10,000-line editor | 0.6255 | 0.3101 | 5.3973 | 2.2421 | 1.5632 | 1.6764 | +7.24% | owner run pending |
| List row hover | 0.3591 | 0.3927 | 43.0472 | 47.0742 | 2.4724 | 2.7143 | +9.78% | owner run pending |
| Auto-scrolling log | 0.5135 | 0.5257 | 4.9206 | 5.0376 | 7.2885 | 7.7386 | +6.18% | owner run pending |
| Card drag | 0.5900 | 0.3075 | 70.7569 | 36.8213 | 1.6264 | 1.5498 | -4.71% | owner run pending |

`damage_cpu` is timed separately and is not added to nested draw/present timing. Signature timing includes any deferred baseline bootstrap; total also includes preflight and cache bookkeeping. The off run performs no signature or planning work.

| Scene, damage on | Signature ms/attempt | Plan ms/attempt | Damage total ms/attempt | Hashed primitives | Reused signatures |
|---|---:|---:|---:|---:|---:|
| Idle caret | 0.006829 | 0.012931 | 0.021792 | 2679 | 114 |
| 10,000-line editor | 0.021606 | 0.040228 | 0.062745 | 458465 | 516 |
| List row hover | 0.009710 | 0.010239 | 0.022852 | 2837614 | 4016 |
| Auto-scrolling log | 0.025769 | 0.094451 | 0.123783 | 530550 | 287 |
| Card drag | 0.006089 | 0.013294 | 0.021108 | 222830 | 7186 |

Frame-mode counts exclude warmup:

| Scene | Damage | Skip | Copy | Partial | Retained | Direct |
|---|---:|---:|---:|---:|---:|---:|
| Idle caret | 0 | 0 | 0 | 0 | 0 | 114 |
| Idle caret | 1 | 0 | 0 | 114 | 0 | 0 |
| 10,000-line editor | 0 | 0 | 0 | 0 | 0 | 518 |
| 10,000-line editor | 1 | 82 | 0 | 434 | 0 | 0 |
| List row hover | 0 | 0 | 0 | 0 | 0 | 7193 |
| List row hover | 1 | 0 | 423 | 0 | 3592 | 3178 |
| Auto-scrolling log | 0 | 0 | 0 | 0 | 0 | 575 |
| Auto-scrolling log | 1 | 0 | 0 | 0 | 288 | 287 |
| Card drag | 0 | 0 | 0 | 0 | 0 | 7197 |
| Card drag | 1 | 1 | 1410 | 5776 | 0 | 0 |

Budget result: **NOT MET**. Acceptance requires damage-on CPU per attempted present to be no more than 3% above off in every scene.
In the four failing scenes, the on-minus-off CPU increase is 0.224 / 0.113 / 0.242 / 0.450 ms per attempt (idle / typing / hover / log), while measured damage work totals 0.022 / 0.063 / 0.023 / 0.124 ms. Reducing only signature and planner work cannot account for the full measured regression. Exact replay also covers relatively few primitives in these workloads; newly painted primitives must still hash. This telemetry does not isolate the remaining difference between scene/provenance work, Metal encoding/copy and run-to-run variation. The candidate therefore does not satisfy G2-7; further CPU profiling is needed before claiming the budget.

G2-5 and G3-1 were not started. The candidate remains WIP and off by default; no G2-6 owner session is ready.

The existing auto-scrolling log prepaint finding remains unfixed: about **5.3 ms per frame** in the initial baseline; this pair measured 5.2907 / 5.4187 ms per prepaint invocation (off/on). The Lab scenes were unchanged.

Reproduce the final candidate from the repository root:

```sh
RUSTUI_DAMAGE=0 ./scripts/run_g0_bench.sh
RUSTUI_DAMAGE=1 ./scripts/run_g0_bench.sh
```

## G2-7 — alternating matched-pair reassessment

Measured 2026-11-21 on the same M1 Max, macOS 27, 1200 × 800 point Retina window and Rust 1.98.0. One optimized release executable with debug symbols was used for all 50 captures, with its hash checked before and after. Compilation and required checks finished before timing; profiling ran afterward. Each capture measured 60 seconds after warmup. No captures had begun under the superseded spread rule.

Each scene has five matched pairs, in this order: off→on, on→off, off→on, on→off, off→on. A run value is mean draw + present elapsed milliseconds per attempted present; the off/on columns are the medians of the five run means. Each paired change is `(on / off − 1) × 100`; its median differs from the ratio of the separate medians. PASS requires all five paired changes ≤3%; FAIL requires all five >3%; any mixture is INCONCLUSIVE. Verdicts use full precision. The old baseline discrepancy is a method warning, not a noise estimate; neither spread nor a median-based exception is used. These results precede the buffered-log fix below.

| Scene | Median off ms/attempt | Median on ms/attempt | Paired changes %, pairs 1–5 | Median change % | Min % | Max % | Verdict |
|---|---:|---:|---|---:|---:|---:|---|
| Idle caret | 2.773821 | 2.832470 | +16.117, -6.580, +2.366, +2.114, +9.060 | +2.366 | -6.580 | +16.117 | **INCONCLUSIVE** |
| 10,000-line editor | 1.669091 | 1.742431 | +6.678, +4.415, +9.515, -1.554, -1.514 | +4.415 | -1.554 | +9.515 | **INCONCLUSIVE** |
| List row hover | 2.523022 | 2.521324 | -5.721, -1.994, +0.078, -1.905, +2.506 | -1.905 | -5.721 | +2.506 | **PASS** |
| Auto-scrolling log | 7.176816 | 7.664600 | +7.643, +10.293, +5.728, +1.010, +1.919 | +5.728 | +1.010 | +10.293 | **INCONCLUSIVE** |
| Card drag | 1.584956 | 1.563019 | +4.304, +3.436, -10.526, +20.401, +17.810 | +4.304 | -10.526 | +20.401 | **INCONCLUSIVE** |

Hover is PASS; the other four scenes are INCONCLUSIVE. None is FAIL. PR #163 remains draft; G2-5 and G3-1 have not started. Stage P remains on hold. No exception to the gate is proposed.

Additional measurements from the same matched captures (median of five run summaries):

| Scene | GPU off ms/s | GPU on ms/s | On signature ms/attempt | On plan ms/attempt | On damage total ms/attempt | Prepaint off / on ms | Power |
|---|---:|---:|---:|---:|---:|---:|---|
| Idle caret | 0.5992 | 0.4166 | 0.006789 | 0.013373 | 0.022115 | 0.4704 / 0.4591 | owner run pending |
| 10,000-line editor | 5.4250 | 2.4723 | 0.021964 | 0.040729 | 0.064019 | 0.7770 / 0.7679 | owner run pending |
| List row hover | 41.9639 | 43.6542 | 0.009772 | 0.010818 | 0.022826 | 2.2265 / 2.1663 | owner run pending |
| Auto-scrolling log | 4.4452 | 4.8767 | 0.025993 | 0.095159 | 0.124784 | 5.0408 / 5.2701 | owner run pending |
| Card drag | 69.1881 | 37.2563 | 0.006330 | 0.013742 | 0.021547 | 0.3312 / 0.3219 | owner run pending |

The original ~5.3 ms log prepaint finding remains recorded without a fix; the Lab scenes were unchanged.

Frame-mode counts summed across the five captures per setting:

| Scene | Damage | Skip | Copy | Partial | Retained | Direct |
|---|---:|---:|---:|---:|---:|---:|
| Idle caret | 0 | 0 | 0 | 0 | 0 | 570 |
| Idle caret | 1 | 0 | 0 | 570 | 0 | 0 |
| 10,000-line editor | 0 | 0 | 0 | 0 | 0 | 2609 |
| 10,000-line editor | 1 | 392 | 0 | 2211 | 0 | 0 |
| List row hover | 0 | 0 | 0 | 0 | 0 | 35609 |
| List row hover | 1 | 0 | 7666 | 0 | 17683 | 10251 |
| Auto-scrolling log | 0 | 0 | 0 | 0 | 0 | 2878 |
| Auto-scrolling log | 1 | 0 | 0 | 0 | 1439 | 1437 |
| Card drag | 0 | 0 | 0 | 0 | 0 | 35719 |
| Card drag | 1 | 5 | 6973 | 28800 | 0 | 0 |

### Samply findings before the log fix

Samply 0.13.1 recorded the four INCONCLUSIVE scenes off and on after the matched pass; hover was already PASS. The log increased in all five pairs, although two increases were ≤3%; the other three scenes had changes in both directions. These extra diagnostics do not establish repeatable regressions in those mixed scenes and do not change any verdict. Each app measured its normal 60-second scene; startup and warmup samples were excluded using its start/end timestamps.

Tables show the ten largest positive **exclusive** function CPU differences in each profile pair, in milliseconds per attempted present. Attribution uses samply’s per-thread CPU deltas (microseconds), weighted at the interval boundaries and summed over app threads; it does not count sleeping wall time as CPU. The deepest resolved inline function receives exclusive time. Normalization uses each profile’s own attempt count. CPU outside draw/present, including input handling, is included in these profile estimates, so they are not the gate metric. These are one-profile-pair estimates, with sampling and attribution uncertainty; positive differences alone do not prove causation. Unresolved leaf addresses are retained rather than assigned invented names.

#### Idle caret

Attempts off/on: 114 / 114. Sampled app CPU off/on: 4.7326 / 5.1585 ms/attempt. Unresolved leaf CPU off/on: 0.2767 / 0.2479 ms/attempt.

| Function | Off ms/attempt | On ms/attempt | Increase ms/attempt |
|---|---:|---:|---:|
| `mach_msg2_trap` | 1.367200 | 1.528399 | +0.161199 |
| `write` | 1.714246 | 1.807105 | +0.092860 |
| `__workq_kernreturn` | 0.965167 | 1.001108 | +0.035942 |
| `<gpui::app::App>::update_entity_erased::{closure#0}` | 0.000000 | 0.031132 | +0.031132 |
| `<core::ptr::non_null::NonNull<alloc::sync::ArcInner<ropey::tree::text::Text>>>::as_ref` | 0.000000 | 0.023140 | +0.023140 |
| `<std::alloc::System as core::alloc::global::GlobalAlloc>::dealloc` | 0.000000 | 0.018693 | +0.018693 |
| `<gpui_base::input::state::InputBaseState<gpui_base::input::kind::InputMode> as gpui::element::Render>::render` | 0.000000 | 0.018123 | +0.018123 |
| `UNRESOLVED libsystem_malloc.dylib+0x37698` | 0.000000 | 0.016895 | +0.016895 |
| `gpui::fast::retained::paint_view::<gpui_component::input::input::Input>` | 0.000000 | 0.016825 | +0.016825 |
| `<gpui::elements::div::Stateful<gpui::elements::div::Div> as gpui::element::Element>::paint` | 0.000000 | 0.014482 | +0.014482 |
#### 10,000-line editor

Attempts off/on: 520 / 519. Sampled app CPU off/on: 13.9426 / 14.1806 ms/attempt. Unresolved leaf CPU off/on: 0.4603 / 0.5299 ms/attempt.

| Function | Off ms/attempt | On ms/attempt | Increase ms/attempt |
|---|---:|---:|---:|
| `ts_tree_cursor_child_iterator_next` | 2.508594 | 2.571842 | +0.063248 |
| `<f32>::total_cmp` | 0.000000 | 0.057863 | +0.057863 |
| `ts_tree_cursor_goto_first_child_internal` | 0.467435 | 0.522734 | +0.055299 |
| `_platform_memmove` | 0.170483 | 0.223345 | +0.052862 |
| `write` | 1.172037 | 1.217514 | +0.045477 |
| `ts_tree_cursor_current_status` | 1.661913 | 1.699175 | +0.037262 |
| `<gpui::scene::Scene>::insert_primitive::<gpui::scene::MonochromeSprite>` | 0.000000 | 0.035064 | +0.035064 |
| `mach_msg2_trap` | 1.346131 | 1.378578 | +0.032448 |
| `<core::num::nonzero::NonZero<u64>>::trailing_zeros` | 0.003681 | 0.032493 | +0.028812 |
| `ts_subtree_summarize_children` | 0.153073 | 0.174682 | +0.021609 |
#### Auto-scrolling log

Attempts off/on: 575 / 575. Sampled app CPU off/on: 7.9071 / 8.2408 ms/attempt. Unresolved leaf CPU off/on: 0.3022 / 0.2940 ms/attempt.

| Function | Off ms/attempt | On ms/attempt | Increase ms/attempt |
|---|---:|---:|---:|
| `write` | 4.863722 | 5.101263 | +0.237541 |
| `gpui::fast::layers::tiles::hash_primitive` | 0.000000 | 0.070918 | +0.070918 |
| `core::intrinsics::likely` | 0.000774 | 0.051238 | +0.050464 |
| `_platform_memmove` | 0.048490 | 0.076242 | +0.027751 |
| `core::slice::sort::shared::smallsort::small_sort_general_with_scratch::<gpui::fast::damage::replay::Entry, <[gpui::fast::damage::replay::Entry]>::sort_by_key<(u32, u32, u32), <gpui::fast::damage::replay::Provenance>::finish::{closure#0}>::{closure#0}>` | 0.000000 | 0.018443 | +0.018443 |
| `core::ptr::copy_nonoverlapping::<u128>` | 0.000000 | 0.015226 | +0.015226 |
| `<gpui::scene::Primitive as core::convert::From<gpui::scene::MonochromeSprite>>::from` | 0.000000 | 0.012986 | +0.012986 |
| `<hashbrown::control::bitmask::BitMask>::lowest_set_bit` | 0.008668 | 0.021024 | +0.012357 |
| `<std::collections::hash::map::HashMap<u64, gpui_apple::fast::damage::plan::Sig, rustc_hash::FxBuildHasher>>::insert` | 0.000000 | 0.011332 | +0.011332 |
| `__kdebug_trace64` | 0.000153 | 0.010803 | +0.010650 |
#### Card drag

Attempts off/on: 7187 / 7194. Sampled app CPU off/on: 2.5560 / 2.6916 ms/attempt. Unresolved leaf CPU off/on: 0.1117 / 0.1313 ms/attempt.

| Function | Off ms/attempt | On ms/attempt | Increase ms/attempt |
|---|---:|---:|---:|
| `write` | 0.876347 | 0.965140 | +0.088794 |
| `__workq_kernreturn` | 0.669712 | 0.683521 | +0.013809 |
| `<taffy::tree::taffy_tree::TaffyView<gpui::taffy::NodeContext, <gpui::taffy::TaffyLayoutEngine>::compute_layout::{closure#1}> as taffy::tree::traits::LayoutPartialTree>::get_core_container_style` | 0.000000 | 0.004475 | +0.004475 |
| `<gpui::window::Window>::mark_view_dirty` | 0.000000 | 0.003796 | +0.003796 |
| `<hashbrown::control::bitmask::BitMask>::lowest_set_bit` | 0.000000 | 0.003133 | +0.003133 |
| `kevent_id` | 0.010645 | 0.013350 | +0.002705 |
| `UNRESOLVED libsystem_malloc.dylib+0x33a48` | 0.002951 | 0.005313 | +0.002362 |
| `<alloc::vec::Vec<core::option::Option<gpui::platform::PlatformInputHandler>>>::len` | 0.000000 | 0.002307 | +0.002307 |
| `core::ptr::write::<u32>` | 0.000000 | 0.002120 | +0.002120 |
| `gpui_apple::fast::damage::plan::signatures_cached` | 0.000000 | 0.001917 | +0.001917 |

Exclusive `write` CPU increased by 0.0455 ms/attempt in typing, 0.2375 in log and 0.0888 in drag; idle’s `write` increase was 0.0929, behind `mach_msg2_trap` (+0.1612). These writes include existing phase recording as well as damage records. Inclusive CPU with damage CPU-record emission on the stack was 0.3589 / 0.1890 ms/attempt for idle (off/on), 0.0564 / 0.0934 for typing, 0.1482 / 0.0819 for log, and 0.1331 / 0.1488 for drag. The self-timed `damage_cpu.total_ms` ends before emitting its record and therefore omits that logging cost. Inclusive figures overlap other functions and must not be added to the exclusive tables.

Other identifiable damage work in the on profiles included the general planner (0.0298 idle, 0.0500 typing, 0.0861 log, 0.0102 drag ms/attempt) and replay provenance (0.0192 typing, 0.0519 log, 0.0020 drag). Signature/cache processing in log accounted for about 0.0215 ms/attempt. These costs are measured; this profile pair does not establish that every cost is unavoidable. Copy-pass encoding and retained-texture management were not separately resolved at this sampling rate, so no numeric claim of an unavoidable copy cost is made.

### Fix and required reassessment

The candidate now formats each damage CPU/GPU/skip/mismatch record in a reused thread-local string, then emits it with one stderr write. The focused test compares the exact bytes with the previous streaming formatter and proves one write per record rather than multiple fragmented writes. Core phase recording, Lab workloads, frame choices and switch defaults are unchanged. The fix is hashed in the existing damage CPU-budget patch and owned sources are regenerated by script. The completed full 50-capture post-fix paired rerun is reported below.

Reproduce matched captures:

```sh
CARGO_PROFILE_RELEASE_DEBUG=1 CARGO_PROFILE_RELEASE_STRIP=none ./scripts/run_g0_bench.sh --interleaved
```

Profile a scene after timing (replace `log` and `0` as needed; this records the complete app and analysis selects its 60-second marked interval):

```sh
RUSTUI_DAMAGE=0 RUSTUI_FRAME_LOG=1 RUSTUI_FRAME_PROFILE=1 samply record --save-only --unstable-presymbolicate --output target/headstart/performance/log-off.json.gz "$HOME/.cache/rustui-target/release/rustui-lab" --g0-scene log --duration 60
```

## G2-7 — post-fix paired retest

Completed all **50 captures** on 2026-11-21 within the approved extension. Measured windows ran from 07:34:01 UTC through 08:27:03 UTC, before the 08:40 UTC cutoff. Every capture completed its 60-second interval and workload validation. The recorded order matched all 50 expected entries, the 1200 × 800 point / scale 2 geometry was identical, and the same post-fix optimized release executable passed its before/after hash check. No compilation, profiling or runtime code changes ran during this retest.

Each scene again used five matched pairs: off→on, on→off, off→on, on→off, off→on. The columns below use the existing draw + present elapsed-time metric per attempted present. Off/on values are medians of five capture means; paired changes are calculated within each pair. PASS requires all five changes ≤3%; FAIL requires all five >3%; any mixture is INCONCLUSIVE. Full precision determines verdicts.

| Scene | Median off ms/attempt | Median on ms/attempt | Paired changes %, pairs 1–5 | Median change % | Min % | Max % | Verdict |
|---|---:|---:|---|---:|---:|---:|---|
| Idle caret | 2.636808 | 2.556394 | -10.178, -13.904, +10.271, +13.747, -3.697 | -3.697 | -13.904 | +13.747 | **INCONCLUSIVE** |
| 10,000-line editor | 1.581541 | 1.615629 | -1.028, +11.368, +5.212, +2.350, +2.137 | +2.350 | -1.028 | +11.368 | **INCONCLUSIVE** |
| List row hover | 2.443804 | 2.495926 | +0.643, +3.965, +4.493, -0.088, -0.904 | +0.643 | -0.904 | +4.493 | **INCONCLUSIVE** |
| Auto-scrolling log | 7.027470 | 7.317259 | +19.653, +3.071, +2.858, +2.530, +5.039 | +3.071 | +2.530 | +19.653 | **INCONCLUSIVE** |
| Card drag | 1.395575 | 1.445623 | +2.652, +4.369, +0.934, +4.382, -2.402 | +2.652 | -2.402 | +4.382 | **INCONCLUSIVE** |

**All five scenes are INCONCLUSIVE.** The all-scenes-PASS gate is not met. PR #163 stays draft; G2-5 and G3-1 remain unstarted, and Stage P stays on hold. Switches remain off by default. No exception is proposed.

**Optional process-CPU diagnostic: skipped.** Exact `getrusage(RUSAGE_SELF)` snapshots after warmup at both measurement boundaries required Lab instrumentation, a rebuild and validation. With about 67 minutes remaining before the cutoff at the start of this turn, the requested captures took priority over that delay risk. Process CPU seconds, seconds per attempt and process-CPU paired changes were not collected; they must not be inferred from these elapsed phase timings.

Capture means used by the paired calculation:

| Scene | Pair | First mode | Off ms/attempt | On ms/attempt | Paired change % |
|---|---:|---|---:|---:|---:|
| Idle caret | 1 | off | 2.636808 | 2.368431 | -10.178 |
| Idle caret | 2 | on | 2.871525 | 2.472264 | -13.904 |
| Idle caret | 3 | off | 2.504919 | 2.762196 | +10.271 |
| Idle caret | 4 | on | 2.467645 | 2.806877 | +13.747 |
| Idle caret | 5 | off | 2.654533 | 2.556394 | -3.697 |
| 10,000-line editor | 1 | off | 1.632407 | 1.615620 | -1.028 |
| 10,000-line editor | 2 | on | 1.527513 | 1.701158 | +11.368 |
| 10,000-line editor | 3 | off | 1.581541 | 1.663966 | +5.212 |
| 10,000-line editor | 4 | on | 1.513603 | 1.549172 | +2.350 |
| 10,000-line editor | 5 | off | 1.581823 | 1.615629 | +2.137 |
| List row hover | 1 | off | 2.443804 | 2.459524 | +0.643 |
| List row hover | 2 | on | 2.427092 | 2.523316 | +3.965 |
| List row hover | 3 | off | 2.388600 | 2.495926 | +4.493 |
| List row hover | 4 | on | 2.523304 | 2.521089 | -0.088 |
| List row hover | 5 | off | 2.517933 | 2.495169 | -0.904 |
| Auto-scrolling log | 1 | off | 6.148298 | 7.356615 | +19.653 |
| Auto-scrolling log | 2 | on | 7.027470 | 7.243269 | +3.071 |
| Auto-scrolling log | 3 | off | 6.906916 | 7.104330 | +2.858 |
| Auto-scrolling log | 4 | on | 7.136706 | 7.317259 | +2.530 |
| Auto-scrolling log | 5 | off | 7.253319 | 7.618803 | +5.039 |
| Card drag | 1 | off | 1.452422 | 1.490943 | +2.652 |
| Card drag | 2 | on | 1.395575 | 1.456546 | +4.369 |
| Card drag | 3 | off | 1.431792 | 1.445161 | +0.934 |
| Card drag | 4 | on | 1.384936 | 1.445623 | +4.382 |
| Card drag | 5 | off | 1.322104 | 1.290342 | -2.402 |

Additional medians from the same captures:

| Scene | GPU off ms/s | GPU on ms/s | On signatures ms/attempt | On plan ms/attempt | On damage total ms/attempt | Prepaint off / on ms | Power |
|---|---:|---:|---:|---:|---:|---:|---|
| Idle caret | 0.5982 | 0.4063 | 0.006083 | 0.014126 | 0.022012 | 0.4770 / 0.4528 | owner run pending |
| 10,000-line editor | 5.2840 | 2.3956 | 0.022463 | 0.040856 | 0.064324 | 0.7656 / 0.7569 | owner run pending |
| List row hover | 42.9760 | 45.4872 | 0.010108 | 0.011279 | 0.023961 | 2.0626 / 2.0176 | owner run pending |
| Auto-scrolling log | 4.6471 | 4.8819 | 0.025891 | 0.096558 | 0.125793 | 5.0144 / 5.1301 | owner run pending |
| Card drag | 69.9520 | 37.8241 | 0.006904 | 0.014577 | 0.023498 | 0.3276 / 0.3234 | owner run pending |

Frame-mode counts summed over the five captures per setting:

| Scene | Damage | Skip | Copy | Partial | Retained | Direct |
|---|---:|---:|---:|---:|---:|---:|
| Idle caret | 0 | 0 | 0 | 0 | 0 | 570 |
| Idle caret | 1 | 0 | 0 | 570 | 0 | 0 |
| 10,000-line editor | 0 | 0 | 0 | 0 | 0 | 2595 |
| 10,000-line editor | 1 | 400 | 0 | 2194 | 0 | 0 |
| List row hover | 0 | 0 | 0 | 0 | 0 | 35715 |
| List row hover | 1 | 0 | 5530 | 0 | 17815 | 12425 |
| Auto-scrolling log | 0 | 0 | 0 | 0 | 0 | 2877 |
| Auto-scrolling log | 1 | 0 | 0 | 0 | 1440 | 1437 |
| Card drag | 0 | 0 | 0 | 0 | 0 | 35760 |
| Card drag | 1 | 5 | 6730 | 28946 | 0 | 0 |

The historical ~5.3 ms log prepaint finding remains recorded without changing the Lab workload. No power sampler, owner verification session, further profiling or new optimization ran in this extension. The source and Metal checks documented before timing were not rerun during this retest-only extension.
