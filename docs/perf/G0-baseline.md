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
