# F0 baseline on gpui-pre

Captured on 2026-11-13 on `f0-1-baselines` before any backend switch. Host: Apple M1 Max, macOS 27.0, Rust 1.98.0. These are GPUI test-window timings, not native display refresh measurements. F0-3…F0-7 must use the same workload, machine, toolchain, and sampling procedure.

| Workload | Samples per run | Run 1 median / p95 | Run 2 median / p95 | Run 3 median / p95 | Baseline median / p95 |
|---|---:|---:|---:|---:|---:|
| 5,000-row Collection scrolling | 300 frames | 11.277 / 12.629 ms | 11.293 / 13.509 ms | 11.202 / 12.157 ms | 11.277 / 12.629 ms |
| 10,000-line Rust CodeEditor typing | 100 edits | 5.775 / 6.972 ms | 5.847 / 14.176 ms | 5.497 / 6.448 ms | 5.775 / 6.972 ms |
| Idle draw with 60 mounted panels | 60 frames | 3.766 / 4.257 ms | 3.702 / 5.210 ms | 3.683 / 4.010 ms | 3.702 / 4.257 ms |
| Existing 1,000-row owner update only | 100 updates per sample; 3 release runs | — | — | — | 4.678 ms |
| Existing 1,000-row owner update + forced draw | 100 updates per sample; 3 release runs | — | — | — | 8.790 ms |

The scrolling workload rebuilt 12,000 visible rows across 300 frames (40 per frame). The idle workload rendered its owner 63 times during setup and the 60 unchanged draws; after one notified state change, the count was 65. The typing workload dispatched 100 simulated characters and drew after each edit. Its second-run p95 outlier is retained in the report rather than filtered.

## Reproduction

```sh
CARGO_TARGET_DIR="$HOME/.cache/rustui-target" rustup run 1.98.0 cargo test -p rustui --lib f0_ --locked --offline -- --ignored --nocapture
CARGO_TARGET_DIR="$HOME/.cache/rustui-target" ./scripts/perf-baseline.sh
```

The F0 workloads are `runtime::f0_benchmarks::{f0_long_list_scroll_benchmark,f0_large_editor_typing_benchmark,f0_idle_redraw_invalidation_benchmark}`. Each workload was run three times; the table reports the median of the three per-run medians and p95 values. The existing release harness reports the median of three runs; its 1,000-row forced-draw spread was 1.74%.
