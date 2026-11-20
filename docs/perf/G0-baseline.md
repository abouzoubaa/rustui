# G0 baseline — GPUI Fast / Metal

Measured 2026-11-20 on an Apple M1 Max (32 GPU cores, 64 GB RAM), macOS 27.0 (26A5425a), Rust 1.98.0 release profile. The built-in Retina display reported 120 Hz. Each scene used a 1200 × 800 point window at scale 2, dark appearance, `RUSTUI_DAMAGE=0`, `RUSTUI_FRAME_LOG=1` and `RUSTUI_FRAME_PROFILE=1`. Compilation and validation finished before capture; no screen capture or power sampler ran during these timings.

Each scene warms for two seconds, then measures 60 seconds. These are one-run native baselines, not repeated-run confidence intervals or battery measurements. Every captured GPU record asserted mode `full`; no partial redraw is implemented in this batch.

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

Run `./scripts/run_g0_bench.sh` from the repository root. It builds the Lab, runs all five scenes sequentially and writes frame logs plus `summary.json` into a new directory under `target/headstart/performance/`. It rejects missing or invalid completion markers, short captures, missing CPU/GPU samples and unexpected GPU modes. Lab without `--g0-scene` still launches the canonical Playground.

The owner runs `./scripts/run_g0_power.sh` from the repository root. It authenticates before measurement, then captures each scene with `sudo powermetrics --samplers gpu_power,cpu_power -i 1000 -n 60`. It saves raw per-scene power samples, the same frame summaries, and CPU/GPU means in `power-summary.json`. Power remains **owner run pending**; no sudo command was run for this baseline. Owner visual review also remains pending.
