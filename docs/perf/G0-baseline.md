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

## G2-8 — implementation fixes, measured in order (WIP)

The instrumented before-E baseline completed all 50 release captures on 2026-11-21. Each scene used five matched pairs, alternating off→on, on→off, off→on, on→off, off→on. Every capture validated its 60-second window, workload and 1200 × 800 point / scale 2 geometry. The executable passed its before/after hash check. No compilation, profiling or source changes ran during timing. Local captures: `target/headstart/performance/g2-8-before-E/`.

`getrusage(RUSAGE_SELF)` snapshots bracket the measured interval after warmup. Process CPU is the user + system delta for the entire process, including worker threads, reported both as seconds per capture and seconds per attempted present. It is diagnostic only. The gate continues to use elapsed draw + present time per attempted present: PASS requires all five paired changes ≤3%; FAIL requires all five >3%; otherwise INCONCLUSIVE. Full precision determines each verdict.

Instance counts measure uploaded scene instances (including path composites), excluding tessellation vertices and fullscreen copy/clear triangles. Totals count the scene before culling. Mode sequences use synchronous encoding records in frame sequence order, rather than asynchronous GPU completion order.

### Before E: paired CPU gate and process-CPU diagnostic

| Scene | Median off ms/attempt | Median on ms/attempt | Five paired changes % | Median / min / max % | Verdict | Process CPU five paired changes % | Process median / min / max % |
|---|---:|---:|---|---|---|---|---|
| idle | 2.484372 | 2.579003 | -8.892, +4.937, +6.736, +8.489, +3.626 | +4.937 / -8.892 / +8.489 | **INCONCLUSIVE** | -8.095, +6.099, +4.615, +5.158, +0.547 | +4.615 / -8.095 / +6.099 |
| typing | 1.530635 | 1.665588 | +4.317, +10.760, +12.593, +9.524, +1.230 | +9.524 / +1.230 / +12.593 | **INCONCLUSIVE** | +2.158, -0.952, +0.283, +1.692, +1.380 | +1.380 / -0.952 / +2.158 |
| hover | 2.470493 | 2.409639 | +1.943, -2.566, -2.696, +3.104, -4.425 | -2.566 / -4.425 / +3.104 | **INCONCLUSIVE** | +1.510, -2.562, -2.305, +2.020, -4.399 | -2.305 / -4.399 / +2.020 |
| log | 7.296455 | 7.427937 | +2.411, +0.649, +1.183, -8.925, +2.218 | +1.183 / -8.925 / +2.411 | **PASS** | +2.152, +2.002, +1.788, -6.862, +2.929 | +2.002 / -6.862 / +2.929 |
| drag | 1.411393 | 1.487519 | +6.439, -0.741, +7.123, +26.382, +18.131 | +7.123 / -0.741 / +26.382 | **INCONCLUSIVE** | +5.575, -0.692, +5.005, +22.259, +13.130 | +5.575 / -0.692 / +22.259 |

Per-capture process CPU (off/on entries are matched within the numbered pair):

| Scene | Pair | Off CPU s | On CPU s | Off CPU s/attempt | On CPU s/attempt |
|---|---:|---:|---:|---:|---:|
| idle | 1 | 0.541434 | 0.497603 | 0.004749421 | 0.004364939 |
| idle | 2 | 0.498121 | 0.528501 | 0.004369482 | 0.004635974 |
| idle | 3 | 0.491513 | 0.514197 | 0.004311518 | 0.004510500 |
| idle | 4 | 0.492037 | 0.517417 | 0.004316114 | 0.004538746 |
| idle | 5 | 0.536000 | 0.538930 | 0.004701754 | 0.004727456 |
| typing | 1 | 7.524536 | 7.686913 | 0.014498143 | 0.014811008 |
| typing | 2 | 7.656276 | 7.598009 | 0.014751977 | 0.014611556 |
| typing | 3 | 7.544722 | 7.580685 | 0.014537037 | 0.014578240 |
| typing | 4 | 7.622171 | 7.736175 | 0.014686264 | 0.014934701 |
| typing | 5 | 7.534630 | 7.594507 | 0.014489673 | 0.014689569 |
| hover | 1 | 22.768778 | 23.109386 | 0.003165408 | 0.003213207 |
| hover | 2 | 22.483966 | 21.901755 | 0.003127551 | 0.003047413 |
| hover | 3 | 22.181387 | 21.661110 | 0.003083746 | 0.003012672 |
| hover | 4 | 22.378141 | 22.858894 | 0.003115431 | 0.003178378 |
| hover | 5 | 22.922618 | 21.892977 | 0.003186352 | 0.003046191 |
| log | 1 | 4.772755 | 4.867020 | 0.008286033 | 0.008464383 |
| log | 2 | 4.893894 | 4.991867 | 0.008496344 | 0.008666436 |
| log | 3 | 4.877950 | 4.973804 | 0.008483391 | 0.008635076 |
| log | 4 | 4.784206 | 4.463640 | 0.008320358 | 0.007749375 |
| log | 5 | 4.844615 | 4.986516 | 0.008425417 | 0.008672202 |
| drag | 1 | 18.478547 | 19.505961 | 0.002568248 | 0.002711421 |
| drag | 2 | 18.747893 | 18.662302 | 0.002611855 | 0.002593788 |
| drag | 3 | 18.614827 | 19.546424 | 0.002587549 | 0.002717045 |
| drag | 4 | 15.757711 | 19.262501 | 0.002189483 | 0.002676834 |
| drag | 5 | 16.239513 | 18.351270 | 0.002256742 | 0.002553043 |

Before-E medians of five capture means; modes and instances are sums across those captures:

| Scene | Damage | GPU ms/s | GPU ms/frame | CPU ms/attempt | Modes skip/copy/partial/retained/direct | Instances encoded / total |
|---|---:|---:|---:|---:|---|---:|
| idle | 0 | 0.611772 | 0.322648 | 2.484372 | 0/0/0/0/570 | 13965 / 13965 |
| idle | 1 | 0.400894 | 0.211481 | 2.579003 | 0/0/570/0/0 | 13965 / 13965 |
| typing | 0 | 5.239152 | 0.605571 | 1.530635 | 0/0/0/0/2596 | 2309146 / 2309146 |
| typing | 1 | 2.380944 | 0.323397 | 1.665588 | 400/0/2194/0/0 | 1951565 / 2307364 |
| hover | 0 | 42.984180 | 0.358799 | 2.470493 | 0/0/0/0/35949 | 14201040 / 14201040 |
| hover | 1 | 45.538508 | 0.379954 | 2.409639 | 0/5223/0/17892/12830 | 12136375 / 14199460 |
| log | 0 | 4.692695 | 0.489638 | 7.296455 | 0/0/0/0/2877 | 2660745 / 2660745 |
| log | 1 | 4.963734 | 0.517919 | 7.427937 | 0/0/0/1440/1438 | 2661668 / 2661668 |
| drag | 0 | 69.747827 | 0.583135 | 1.411393 | 0/0/0/0/35958 | 1150720 / 1150720 |
| drag | 1 | 35.809381 | 0.298712 | 1.487519 | 5/6466/29492/0/0 | 943776 / 1150944 |

Before-E synchronous mode sequences, damage on (pair 1):

| Scene | First 12 mode runs (mode × frames) | Total runs | Retained↔Direct transitions |
|---|---|---:|---:|
| log | retained×1, direct×1, retained×1, direct×1, retained×1, direct×1, retained×1, direct×1, retained×1, direct×1, retained×1, direct×1 | 575 | 574 |
| hover | direct×1, retained×1, direct×1, retained×1, direct×1, retained×1, copy×1, retained×1, direct×1, retained×1, copy×1, retained×1 | 7192 | 6405 |

The before-E gate is not met: log is PASS and the other four scenes are INCONCLUSIVE. These are baseline verdicts, not acceptance of the subsequent fixes. Power remains **owner run pending**. The previously recorded ~5.3 ms log CPU prepaint finding is unchanged; the workload has not been altered.

### E: Direct policy

A new candidate patch compares scene metadata even when Direct has invalidated retained pixels. Unsupported frames preserve the large-frame history; large planned changes choose Direct immediately. Re-priming after large changes requires two consecutive cheap comparable frames, avoiding a full redraw plus copy between large frames while allowing recovery to Copy/Skip/Partial. Kept-texture lifetime, prime scene snapshots, cache writes and Partial culling remain separate fixes.

Post-E retesting is now **complete: 50 captures**. G2-8 collected the first 34; G2-8a resumed the missing 16 (typing pairs 3–5 and drag pairs 1–5), without rebuilding. The executable passed its hash checks before and after the resumed segment. Each scene preserved off→on, on→off, off→on, on→off, off→on. All 50 timed windows, workloads, geometry and getrusage measurements validated. The interrupted third typing pair from G2-8 is preserved separately and excluded. No compilation, profiling or runtime source changes ran during timing. Captures: `target/headstart/performance/g2-8-after-E/`.

### E before/after (damage on)

Values are medians of five capture means; mode and instance counts are sums. Arrows show before E → after E. The paired table supplies contemporaneous off controls; absolute medians across passes alone do not establish a CPU gain. Mode order is skip/copy/partial/retained/direct.

| Scene | GPU ms/s | GPU ms/frame | CPU ms/attempt | Mode counts | Instances encoded / total |
|---|---:|---:|---:|---|---|
| idle | 0.400894 → 0.399529 | 0.211481 → 0.210792 | 2.579003 → 2.587903 | 0/0/570/0/0 → 0/0/570/0/0 | 13965 / 13965 → 13965 / 13965 |
| typing | 2.380944 → 2.462415 | 0.323397 → 0.337043 | 1.665588 → 1.538319 | 400/0/2194/0/0 → 397/0/2201/0/0 | 1951565 / 2307364 → 1957792 / 2310929 |
| hover | 45.538508 → 43.784724 | 0.379954 → 0.365535 | 2.409639 → 2.564250 | 0/5223/0/17892/12830 → 0/52/0/142/35757 | 12136375 / 14199460 → 14180895 / 14201435 |
| log | 4.963734 → 4.550980 | 0.517919 → 0.474949 | 7.427937 → 7.437110 | 0/0/0/1440/1438 → 0/0/0/0/2876 | 2661668 / 2661668 → 2659786 / 2659786 |
| drag | 35.809381 → 36.105785 | 0.298712 → 0.304392 | 1.487519 → 1.473315 | 5/6466/29492/0/0 → 5/6576/29228/0/0 | 943776 / 1150944 → 935296 / 1145984 |

### Post-E paired CPU gate and process-CPU diagnostic

| Scene | Median off / on ms/attempt | Five paired CPU changes % | CPU median / min / max % | Verdict | Process CPU five paired changes % | Process median / min / max % |
|---|---:|---|---|---|---|---|
| idle | 2.505412 / 2.587903 | +3.200, +16.145, +0.042, +3.293, +29.595 | +3.293 / +0.042 / +29.595 | **INCONCLUSIVE** | +0.642, +10.313, -1.510, +0.976, +19.284 | +0.976 / -1.510 / +19.284 |
| typing | 1.503467 / 1.538319 | -2.189, +5.017, +11.377, -0.939, +0.244 | +0.244 / -2.189 / +11.377 | **INCONCLUSIVE** | -4.806, +0.024, +1.126, +3.855, -1.264 | +0.024 / -4.806 / +3.855 |
| hover | 2.593630 / 2.564250 | -4.416, -6.999, -1.123, -3.233, +0.447 | -3.233 / -6.999 / +0.447 | **PASS** | -3.749, -5.793, -1.499, -2.978, +0.418 | -2.978 / -5.793 / +0.418 |
| log | 7.029334 / 7.437110 | +20.061, +1.127, +5.801, +17.862, +3.656 | +5.801 / +1.127 / +20.061 | **INCONCLUSIVE** | +16.955, +0.449, +5.576, +15.194, +3.028 | +5.576 / +0.449 / +16.955 |
| drag | 1.409434 / 1.473315 | +7.460, +7.923, +4.334, +3.537, +3.949 | +4.334 / +3.537 / +7.923 | **FAIL** | +5.539, +7.275, +4.399, +3.835, +3.711 | +4.399 / +3.711 / +7.275 |

Every post-E capture, including the 16 completed in G2-8a. Switch is RUSTUI_DAMAGE. Process CPU is user + system measured after warmup; its paired changes are diagnostic only.

| Scene | Pair | Switch | Process CPU user+sys s | Process CPU s/attempt | GPU ms/s | GPU ms/frame | CPU ms/attempt | Modes skip/copy/partial/retained/direct | Instances encoded / total |
|---|---:|---:|---:|---:|---:|---:|---:|---|---|
| idle | 1 | 0 | 0.534439 | 0.004688061 | 0.595843 | 0.313943 | 2.647204 | 0/0/0/0/114 | 2793 / 2793 |
| idle | 1 | 1 | 0.537868 | 0.004718140 | 0.388967 | 0.205213 | 2.731921 | 0/0/114/0/0 | 2793 / 2793 |
| idle | 2 | 0 | 0.491913 | 0.004315026 | 0.625207 | 0.329954 | 2.374830 | 0/0/0/0/114 | 2793 / 2793 |
| idle | 2 | 1 | 0.542642 | 0.004760018 | 0.399529 | 0.210792 | 2.758254 | 0/0/114/0/0 | 2793 / 2793 |
| idle | 3 | 0 | 0.514895 | 0.004516623 | 0.605059 | 0.319293 | 2.570572 | 0/0/0/0/114 | 2793 / 2793 |
| idle | 3 | 1 | 0.507121 | 0.004448430 | 0.368314 | 0.194163 | 2.571648 | 0/0/114/0/0 | 2793 / 2793 |
| idle | 4 | 0 | 0.504353 | 0.004424149 | 0.594662 | 0.313347 | 2.505412 | 0/0/0/0/114 | 2793 / 2793 |
| idle | 4 | 1 | 0.509274 | 0.004467316 | 0.486952 | 0.256792 | 2.587903 | 0/0/114/0/0 | 2793 / 2793 |
| idle | 5 | 0 | 0.426027 | 0.003737079 | 0.639523 | 0.337243 | 1.987402 | 0/0/0/0/114 | 2793 / 2793 |
| idle | 5 | 1 | 0.508183 | 0.004457746 | 0.430156 | 0.226808 | 2.575579 | 0/0/114/0/0 | 2793 / 2793 |
| typing | 1 | 0 | 7.408480 | 0.014219731 | 5.373511 | 0.619262 | 1.503467 | 0/0/0/0/521 | 463430 / 463430 |
| typing | 1 | 1 | 7.065955 | 0.013536312 | 2.498045 | 0.337043 | 1.470555 | 77/0/445/0/0 | 395828 / 464322 |
| typing | 2 | 0 | 7.511091 | 0.014500176 | 5.240802 | 0.608108 | 1.435976 | 0/0/0/0/518 | 460761 / 460761 |
| typing | 2 | 1 | 7.512925 | 0.014503716 | 2.462415 | 0.338660 | 1.508025 | 81/0/437/0/0 | 388712 / 460768 |
| typing | 3 | 0 | 7.507878 | 0.014466046 | 5.330071 | 0.617068 | 1.456250 | 0/0/0/0/519 | 461648 / 461648 |
| typing | 3 | 1 | 7.592424 | 0.014628948 | 2.356791 | 0.322630 | 1.621928 | 80/0/439/0/0 | 390491 / 461651 |
| typing | 4 | 0 | 7.405406 | 0.014186602 | 5.246136 | 0.604184 | 1.587492 | 0/0/0/0/522 | 464310 / 464310 |
| typing | 4 | 1 | 7.631949 | 0.014733492 | 2.364606 | 0.325145 | 1.572583 | 81/0/437/0/0 | 388712 / 460755 |
| typing | 5 | 0 | 7.554371 | 0.014527637 | 5.354299 | 0.618575 | 1.534573 | 0/0/0/0/520 | 462538 / 462538 |
| typing | 5 | 1 | 7.473226 | 0.014344004 | 2.502475 | 0.339634 | 1.538319 | 78/0/443/0/0 | 394049 / 463433 |
| hover | 1 | 0 | 23.051472 | 0.003205155 | 43.474456 | 0.362789 | 2.569748 | 0/0/0/0/7191 | 2840840 / 2840840 |
| hover | 1 | 1 | 22.196439 | 0.003084981 | 43.847840 | 0.365705 | 2.456262 | 0/25/0/67/7103 | 2832150 / 2842025 |
| hover | 2 | 0 | 23.201567 | 0.003227820 | 43.443800 | 0.362739 | 2.593630 | 0/0/0/0/7187 | 2839260 / 2839260 |
| hover | 2 | 1 | 21.881731 | 0.003040819 | 43.555806 | 0.363192 | 2.412108 | 0/24/0/45/7127 | 2832940 / 2842420 |
| hover | 3 | 0 | 23.119775 | 0.003216888 | 44.041844 | 0.367751 | 2.593368 | 0/0/0/0/7186 | 2838865 / 2838865 |
| hover | 3 | 1 | 22.744612 | 0.003168656 | 43.784724 | 0.366008 | 2.564250 | 0/2/0/6/7170 | 2834520 / 2835310 |
| hover | 4 | 0 | 23.607379 | 0.003282906 | 42.699204 | 0.356278 | 2.650040 | 0/0/0/0/7191 | 2840445 / 2840445 |
| hover | 4 | 1 | 22.904321 | 0.003185137 | 43.802187 | 0.365535 | 2.564355 | 0/1/0/23/7166 | 2840050 / 2840445 |
| hover | 5 | 0 | 23.425518 | 0.003257163 | 43.097741 | 0.359564 | 2.636814 | 0/0/0/0/7192 | 2840840 / 2840840 |
| hover | 5 | 1 | 23.526591 | 0.003270762 | 43.503561 | 0.362996 | 2.648588 | 0/0/0/1/7191 | 2841235 / 2841235 |
| log | 1 | 0 | 4.106828 | 0.007129910 | 4.831776 | 0.504149 | 5.947316 | 0/0/0/0/576 | 532701 / 532701 |
| log | 1 | 1 | 4.794797 | 0.008338777 | 4.659057 | 0.486213 | 7.140409 | 0/0/0/0/575 | 531778 / 531778 |
| log | 2 | 0 | 4.831020 | 0.008387187 | 4.577213 | 0.477589 | 7.201485 | 0/0/0/0/576 | 532701 / 532701 |
| log | 2 | 1 | 4.852692 | 0.008424813 | 4.269667 | 0.445496 | 7.282641 | 0/0/0/0/576 | 532701 / 532701 |
| log | 3 | 0 | 4.700177 | 0.008174221 | 4.542806 | 0.474237 | 7.029334 | 0/0/0/0/575 | 531805 / 531805 |
| log | 3 | 1 | 4.962259 | 0.008630016 | 4.550980 | 0.474949 | 7.437110 | 0/0/0/0/575 | 531769 / 531769 |
| log | 4 | 0 | 4.301576 | 0.007481002 | 4.819466 | 0.503040 | 6.359753 | 0/0/0/0/575 | 531769 / 531769 |
| log | 4 | 1 | 4.955143 | 0.008617640 | 4.549972 | 0.474806 | 7.495742 | 0/0/0/0/575 | 531769 / 531769 |
| log | 5 | 0 | 4.839858 | 0.008417144 | 4.669627 | 0.487562 | 7.239049 | 0/0/0/0/575 | 531769 / 531769 |
| log | 5 | 1 | 4.986419 | 0.008672033 | 4.583790 | 0.478372 | 7.503735 | 0/0/0/0/575 | 531769 / 531769 |
| drag | 1 | 0 | 16.825502 | 0.002359156 | 68.598181 | 0.577202 | 1.295136 | 0/0/0/0/7132 | 228224 / 228224 |
| drag | 1 | 1 | 17.896869 | 0.002489826 | 36.785975 | 0.307177 | 1.391755 | 1/1357/5829/0/0 | 186528 / 230016 |
| drag | 2 | 0 | 17.629451 | 0.002449556 | 69.767458 | 0.581747 | 1.365149 | 0/0/0/0/7196 | 230304 / 230304 |
| drag | 2 | 1 | 18.709622 | 0.002627756 | 36.105785 | 0.304392 | 1.473315 | 1/1253/5865/0/0 | 187680 / 227840 |
| drag | 3 | 0 | 18.108904 | 0.002518624 | 69.214030 | 0.577618 | 1.409434 | 0/0/0/0/7190 | 230080 / 230080 |
| drag | 3 | 1 | 18.710878 | 0.002629409 | 35.004394 | 0.295232 | 1.470520 | 1/1413/5702/0/0 | 182464 / 227712 |
| drag | 4 | 0 | 18.539559 | 0.002581752 | 68.483772 | 0.572328 | 1.440651 | 0/0/0/0/7180 | 229792 / 229792 |
| drag | 4 | 1 | 19.282706 | 0.002680760 | 36.074602 | 0.301014 | 1.491603 | 1/1270/5921/0/0 | 189472 / 230176 |
| drag | 5 | 0 | 18.378104 | 0.002556065 | 69.193320 | 0.577473 | 1.418709 | 0/0/0/0/7190 | 230080 / 230080 |
| drag | 5 | 1 | 19.073448 | 0.002650931 | 36.979704 | 0.308465 | 1.474734 | 1/1283/5911/0/0 | 189152 / 230240 |

Post-E synchronous mode sequences (pair 1, damage on):

| Scene | First 12 runs (mode × frames) | Total runs | Retained↔Direct transitions |
|---|---|---:|---:|
| log | direct×575 | 1 | 0 |
| hover | direct×20, retained×1, copy×1, direct×124, retained×1, direct×83, retained×1, copy×1, direct×82, retained×1, copy×1, direct×82 | 160 | 109 |

All five post-E damage-on log sequences contain only Direct: **2,876 Direct, zero Retained**. Hover Retained frames fell from 17,892 to 142. E's all-scene CPU gate is not met: hover PASS; idle, typing and log INCONCLUSIVE; drag FAIL. All five drag paired changes exceed 3%. No gate exception is proposed.

The unresolved E tradeoff is log planner time: 0.095155 → 0.189336 ms/attempt. Metadata comparison after Direct runs the planner every plannable frame, rather than alternating with whole-frame priming. Log signature time is 0.025072 → 0.025051 ms/attempt. F below addresses the repeated planning; allocation reuse, clone removal and cache-write changes remain for G2-8c.

## G2-8a — F planner back-off

`RUSTUI_DAMAGE_BACKOFF=1` opts in; it defaults off. After eight consecutive completed plannable frames ending Direct or large, the next 30 completed frames draw Direct without signatures or planning. The following frame probes once; a large/Direct probe starts another 30-frame run. Unsupported→plannable changes, resize, scale and atlas loss reset the counters. Off paths and failed drawable acquisition do not advance the completed-frame counter.

`damage_cpu.reason` and synchronous encode records report `backoff`; actual frame mode remains Direct. Summaries preserve both rendering-mode runs and planning runs, plus back-off frame counts.

Recovery is bounded in frame intervals, preserving the requested 30 skipped frames: indices 0–29 draw Direct, index 30 primes retained pixels if the adjacent scene change is small, and index 31 can be Partial. That is 31 intervals from the first small-change frame, or 32 rendered frames counting it. On the last skipped frame, an unhashed scene snapshot supplies an adjacent-frame comparison for the probe; an older snapshot could incorrectly restart back-off. Its cost is included in measured CPU and remains visible for the later clone-removal work. Headless tests assert no planner calls in every 30-frame run, exactly one probe, counter resets, true Partial recovery with magenta pre-fill, and byte equality with whole draws throughout.

Reproduce F's comparison (one release build, damage enabled in both arms):

```sh
CARGO_PROFILE_RELEASE_DEBUG=1 CARGO_PROFILE_RELEASE_STRIP=none ./scripts/run_g0_bench.sh --backoff-pairs target/headstart/performance/g2-8a-backoff-repeat
```


### F paired results

Each scene used five matched pairs with the same alternating order; damage stays 1, and switch 0/1 means RUSTUI_DAMAGE_BACKOFF. All 20 windows validated, and the single executable passed its before/after hash check. No compilation or profiling ran during timing. Process CPU remains diagnostic; verdicts use the unchanged phase-CPU rule.

| Scene | Median off / on ms/attempt | Five paired CPU changes % | CPU median / min / max % | Verdict | Process CPU five paired changes % | Process median / min / max % |
|---|---:|---|---|---|---|---|
| log | 7.426991 / 7.019087 | -4.941, -4.186, -5.492, -5.039, -5.335 | -5.039 / -5.492 / -4.186 | **PASS** | -2.750, -4.015, -4.416, -4.205, -3.635 | -4.015 / -4.416 / -2.750 |
| hover | 2.524198 / 2.546558 | +1.404, -1.493, +1.853, +2.311, -0.924 | +1.404 / -1.493 / +2.311 | **PASS** | +1.477, -1.135, +1.402, +1.852, -0.716 | +1.402 / -1.135 / +1.852 |

Medians of five capture means; counts are sums:

| Scene | Back-off | Planner ms/attempt | Damage total ms/attempt | GPU ms/s | GPU ms/frame | Back-off frames | Modes | Instances encoded / total |
|---|---:|---:|---:|---:|---:|---:|---|---|
| log | 0 | 0.191672 | 0.218540 | 4.481354 | 0.467621 | 0 | 0/0/0/0/2878 | 2661650 / 2661650 |
| log | 1 | 0.007466 | 0.010594 | 4.548670 | 0.474751 | 2755 | 0/0/0/0/2875 | 2658854 / 2658854 |
| hover | 0 | 0.016720 | 0.027352 | 43.832370 | 0.365832 | 0 | 0/71/0/166/35716 | 14173785 / 14201830 |
| hover | 1 | 0.001692 | 0.003884 | 43.828993 | 0.365608 | 32501 | 0/11/0/295/35645 | 14197485 / 14201830 |

F reduces log planning and process CPU, while measured GPU time is 4.481354 → 4.548670 ms/s. Hover GPU time is nearly unchanged (43.832370 → 43.828993 ms/s), and its median paired phase/process CPU changes are both about +1.4%, within the 3% rule. Hover Retained frames increase from 166 to 295 and Copy frames fall from 71 to 11; F does not reduce instance uploads. These measurements do not establish a power saving or authorize default-on. Power remains owner run pending.

Every F capture; switch is RUSTUI_DAMAGE_BACKOFF, with damage=1 throughout.

| Scene | Pair | Switch | Process CPU user+sys s | Process CPU s/attempt | GPU ms/s | GPU ms/frame | CPU ms/attempt | Modes skip/copy/partial/retained/direct | Instances encoded / total |
|---|---:|---:|---:|---:|---:|---:|---:|---|---|
| log | 1 | 0 | 4.907031 | 0.008533967 | 4.478685 | 0.467493 | 7.451605 | 0/0/0/0/575 | 531769 / 531769 |
| log | 1 | 1 | 4.772096 | 0.008299297 | 4.642811 | 0.484521 | 7.083440 | 0/0/0/0/575 | 531769 / 531769 |
| log | 2 | 0 | 4.975880 | 0.008638681 | 4.481354 | 0.467621 | 7.432730 | 0/0/0/0/576 | 532701 / 532701 |
| log | 2 | 1 | 4.767790 | 0.008291809 | 4.548670 | 0.474751 | 7.121609 | 0/0/0/0/575 | 531769 / 531769 |
| log | 3 | 0 | 4.943392 | 0.008597203 | 4.484196 | 0.467985 | 7.426991 | 0/0/0/0/575 | 531769 / 531769 |
| log | 3 | 1 | 4.725107 | 0.008217577 | 4.544355 | 0.474309 | 7.019087 | 0/0/0/0/575 | 531778 / 531778 |
| log | 4 | 0 | 4.782792 | 0.008303458 | 4.160189 | 0.434106 | 7.093676 | 0/0/0/0/576 | 532701 / 532701 |
| log | 4 | 1 | 4.573742 | 0.007954334 | 4.589391 | 0.478924 | 6.736208 | 0/0/0/0/575 | 531769 / 531769 |
| log | 5 | 0 | 4.846387 | 0.008413866 | 4.523893 | 0.472057 | 7.396973 | 0/0/0/0/576 | 532710 / 532710 |
| log | 5 | 1 | 4.662092 | 0.008107986 | 4.534298 | 0.473211 | 7.002312 | 0/0/0/0/575 | 531769 / 531769 |
| hover | 1 | 0 | 22.720690 | 0.003158722 | 43.824935 | 0.365574 | 2.533797 | 0/4/0/15/7174 | 2839655 / 2841235 |
| hover | 1 | 1 | 23.059581 | 0.003205391 | 43.828993 | 0.365608 | 2.569380 | 0/2/0/47/7145 | 2840840 / 2841630 |
| hover | 2 | 0 | 22.680326 | 0.003151796 | 44.102241 | 0.367733 | 2.524198 | 0/5/0/9/7182 | 2840445 / 2842420 |
| hover | 2 | 1 | 22.422912 | 0.003116024 | 44.181098 | 0.368439 | 2.486502 | 0/0/0/51/7145 | 2842420 / 2842420 |
| hover | 3 | 0 | 22.694885 | 0.003156451 | 43.790123 | 0.365479 | 2.521890 | 0/16/0/57/7117 | 2833730 / 2840050 |
| hover | 3 | 1 | 22.974722 | 0.003200714 | 43.389108 | 0.362766 | 2.568626 | 0/3/0/65/7109 | 2834125 / 2835310 |
| hover | 4 | 0 | 22.124475 | 0.003076690 | 43.832370 | 0.365832 | 2.444700 | 0/23/0/37/7130 | 2831360 / 2840445 |
| hover | 4 | 1 | 22.534137 | 0.003133658 | 43.801312 | 0.365546 | 2.501191 | 0/4/0/67/7119 | 2838865 / 2840445 |
| hover | 5 | 0 | 23.066341 | 0.003210794 | 44.327602 | 0.370259 | 2.570303 | 0/23/0/48/7113 | 2828595 / 2837680 |
| hover | 5 | 1 | 22.936172 | 0.003187793 | 44.023700 | 0.367183 | 2.546558 | 0/2/0/65/7127 | 2841235 / 2842025 |

Synchronous planning sequence, F on, pair 1:

| Scene | First 12 planning runs | Back-off frames |
|---|---|---:|
| log | direct×6, backoff×30, direct×1, backoff×30, direct×1, backoff×30, direct×1, backoff×30, direct×1, backoff×30, direct×1, backoff×30 | 551 |
| hover | direct×7, backoff×30, direct×1, backoff×30, direct×1, backoff×30, direct×1, backoff×30, direct×1, backoff×30, retained×1, direct×8 | 6590 |

G2-8a stops here. G2-8b/c/d remain unstarted pending the owner; G2-5/G3-1 remain unstarted, PR #163 remains draft, and Stage P stays on hold. Power is owner run pending. These two F scenes do not replace G2-8d’s final all-scene gate.

## PX-1 — buffered recorder observer proof

Measured on the same M1 Max/macOS 27 host with one pinned Rust 1.98.0 release build. Five matched log-scene pairs alternate phase profiling off→on, on→off, off→on, on→off, off→on. Both arms use `RUSTUI_DAMAGE=0` and `RUSTUI_FRAME_LOG=buffered`; only `RUSTUI_FRAME_PROFILE` changes. Every capture measures 60 seconds after the normal warmup. `getrusage(RUSAGE_SELF)` user + system snapshots bracket the measured window; startup, cleanup and exit serialization are excluded. No compiler, profiler or other benchmark was running; the background repository indexer was paused during timing. macOS background services remained active. This is an observer comparison, not an application optimization claim.

An earlier set using external process counters was stopped after the owner specified `getrusage` as the deciding metric. None of that set contributes to these numbers or verdicts. Raw logs and summaries remain local and ignored under `target/headstart/px1/paired-rusage/`.

The candidate stores raw frame records and preserves the original field order, precision, null spelling and newline. `RUSTUI_FRAME_LOG=1` drains after present, outside phase timers; `buffered` formats and drains at normal process exit. Focused tests compare exact bytes against the original formatter and prove the exit hook drains even through `process::exit`. GPU timing stays enabled in buffered mode. All damage switches remain off by default; no Fix F threshold changes.

| Pair | Order | Process CPU off ms/attempt | On ms/attempt | Paired change | Draw+present off / on ms/attempt | Frames >8.3 ms off / on |
|---|---|---:|---:|---:|---:|---:|
| 1 | off→on | 3.380840 | 3.417625 | +1.088% | 1.777696 / 1.777321 | 0 / 0 |
| 2 | on→off | 3.378007 | 3.450727 | +2.153% | 1.826418 / 1.874562 | 0 / 0 |
| 3 | off→on | 3.385579 | 3.355270 | -0.895% | 1.852305 / 1.835725 | 0 / 0 |
| 4 | on→off | 3.686132 | 3.596082 | -2.443% | 2.129474 / 2.072211 | 0 / 0 |
| 5 | off→on | 3.801889 | 3.734957 | -1.761% | 2.209206 / 2.138347 | 0 / 0 |

Median process CPU off / on: **3.385579 / 3.450727 ms/attempt**. Paired median / min / max: **-0.895% / -2.443% / +2.153%**. Stage P verdict: **NEUTRAL**, using [the owner rule](MEASUREMENT_RULES.md). PX-1's separate ≤5% observer-overhead requirement: **PASS**. Key/append-to-screen delay: **not measured; PX-2 pending**.

| Pair | Profile | User CPU s | System CPU s | Total CPU s | Attempts | GPU ms/frame | GPU ms/s | Prepaint ms/call |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 0 | 1.370732000 | 0.573251000 | 1.943983000 | 575 | 0.504685 | 4.836310 | not recorded |
| 1 | 1 | 1.378890000 | 0.589662000 | 1.968552000 | 576 | 0.542030 | 5.194452 | 0.490265 |
| 2 | 0 | 1.379380000 | 0.562974000 | 1.942354000 | 575 | 0.502533 | 4.815827 | not recorded |
| 2 | 1 | 1.411164000 | 0.573004000 | 1.984168000 | 575 | 0.514463 | 4.930020 | 0.513523 |
| 3 | 0 | 1.383690000 | 0.563018000 | 1.946708000 | 575 | 0.506298 | 4.851338 | not recorded |
| 3 | 1 | 1.372936000 | 0.556344000 | 1.929280000 | 575 | 0.527306 | 5.052199 | 0.506210 |
| 4 | 0 | 1.513512000 | 0.606014000 | 2.119526000 | 575 | 0.473568 | 4.537727 | not recorded |
| 4 | 1 | 1.483372000 | 0.584375000 | 2.067747000 | 575 | 0.485069 | 4.647570 | 0.578548 |
| 5 | 0 | 1.563271000 | 0.626617000 | 2.189888000 | 576 | 0.488481 | 4.681360 | not recorded |
| 5 | 1 | 1.536773000 | 0.610827000 | 2.147600000 | 575 | 0.514283 | 4.925055 | 0.588517 |

## Post-PX-1 baseline

Recorded after PR #165 merged and PX-1 propagated to both requested branches, from `g-integration` at `8ade25a526f105cd5d6f1f185d85ce83fbfd5454`. Same Apple M1 Max / 64 GB / macOS 27.0 (26A5425a), pinned Rust 1.98.0 release build, 1200 × 800 point dark window at scale 2. One 60-second capture per scene followed the normal warmup, with `RUSTUI_DAMAGE=0`, `RUSTUI_FRAME_LOG=buffered` and `RUSTUI_FRAME_PROFILE=1`. The build and all local checks finished before timing; no compiler, profiler or other benchmark ran during capture. The background repository indexer was paused for capture and resumed afterward; normal macOS background services remained active.

These are fresh single-run baselines, not paired comparisons or Stage P verdicts. They replace the observer-heavy baseline for subsequent comparisons; they do not retrospectively change any earlier G2 verdict. `getrusage(RUSAGE_SELF)` brackets each measured window. Process CPU includes user + system time across all process threads; draw + present is elapsed phase time. GPU times use completed Metal command buffers. Powers remain **owner run pending**. Input-to-screen latency is **not measured; PX-2 pending**.

Raw captures and `summary.json` are local ignored artifacts under `target/headstart/px1/post-px1-baseline/`. Reproduce with `RUSTUI_DAMAGE=0 ./scripts/run_g0_bench.sh`.

| Scene | GPU mean / p95 ms/frame | GPU ms/s | Draw+present ms/attempt | Process CPU ms/attempt | Frames >8.3 ms |
|---|---:|---:|---:|---:|---:|
| idle | 0.333809 / 0.623000 | 0.632712 | 0.815695 | 2.669070 | 0 |
| typing | 0.657617 / 1.106834 | 5.706824 | 0.848342 | 13.576631 | 0 |
| hover | 0.370759 / 0.636750 | 44.475327 | 1.765818 | 2.666216 | 0 |
| log | 0.509569 / 0.822500 | 4.882806 | 2.311944 | 3.954327 | 0 |
| drag | 0.720489 / 0.923000 | 86.410682 | 0.418664 | 1.490406 | 0 |

Frames >8.3 ms count draw + present elapsed phase time; they are not presentation-gap or screen-delay counts. CPU and GPU boundary counts can differ because GPU completion arrives asynchronously.

| Scene | Measured s | CPU attempts | User CPU s | System CPU s | Total CPU s | Modes skip/copy/partial/retained/direct |
|---|---:|---:|---:|---:|---:|---|
| idle | 60.144625 | 114 | 0.169211000 | 0.135063000 | 0.304274000 | 0/0/0/0/114 |
| typing | 60.036586 | 521 | 6.535040000 | 0.538385000 | 7.073425000 | 0/0/0/0/521 |
| hover | 60.004645 | 7199 | 14.493547000 | 4.700544000 | 19.194091000 | 0/0/0/0/7198 |
| log | 60.006891 | 575 | 1.633940000 | 0.639798000 | 2.273738000 | 0/0/0/0/575 |
| drag | 60.008279 | 7198 | 5.380833000 | 5.347112000 | 10.727945000 | 0/0/0/0/7197 |

CPU phase means below are elapsed milliseconds per phase invocation. Phases nest and must not be added together.

| Scene | Setup | Build | Layout | Prepaint | Paint | Draw | Present |
|---|---:|---:|---:|---:|---:|---:|---:|
| idle | 0.003824 | 0.327727 | 0.006935 | 0.149224 | 0.081261 | 0.627228 | 0.188467 |
| typing | 0.001859 | 0.155823 | 0.001233 | 0.433094 | 0.106789 | 0.748865 | 0.099819 |
| hover | 0.002671 | 1.117590 | 0.000793 | 0.602588 | 0.255045 | 2.070477 | 0.145732 |
| log | 0.003659 | 0.168628 | 0.006836 | 0.609075 | 1.183095 | 2.114717 | 0.197227 |
| drag | 0.003058 | 0.107733 | 0.043664 | 0.096671 | 0.059061 | 0.302958 | 0.170675 |
