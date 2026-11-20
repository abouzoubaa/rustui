# F0-10 benchmark method and reproducibility

Captured 2026-11-14 on Apple M1 Max, macOS 27.0, Rust 1.98.0. The machine was on AC power (pmset -g batt reported “Now drawing from 'AC Power'”; AC powermode 0, sleep 0). system_profiler SPPowerDataType reported Current Power Source: Yes and AC Low Power Mode: No. No Cargo build ran during timed runs.

## Exact F0-10 build and run procedure

The gpui-pre baseline is a clean worktree at 925436b51e3f226c3057bfad810f0780328b239a (/private/tmp/f0-before-backend). Fast is f0-integration commit 4b27888b0dbad6f8212d2ca62b8438db2c81e229. Benchmark source files and scripts/perf-baseline.sh are byte-identical at those revisions. Each backend has its own target directory:

| Backend | CARGO_TARGET_DIR |
|---|---|
| gpui-pre | $HOME/.cache/rustui-target-f0-10-gpui-pre |
| GPUI Fast | $HOME/.cache/rustui-target-f0-10-gpui-fast |

Before timing, each backend was built sequentially in both profiles, with RUSTC and RUSTDOC set to the Rust 1.98.0 toolchain binaries:

    env RUSTC="$(rustup which --toolchain 1.98.0 rustc)" RUSTDOC="$(rustup which --toolchain 1.98.0 rustdoc)" CARGO_TARGET_DIR="$HOME/.cache/rustui-target-f0-10-gpui-pre" rustup run 1.98.0 cargo test -p rustui --lib --no-run --locked --offline
    env RUSTC="$(rustup which --toolchain 1.98.0 rustc)" RUSTDOC="$(rustup which --toolchain 1.98.0 rustdoc)" CARGO_TARGET_DIR="$HOME/.cache/rustui-target-f0-10-gpui-pre" rustup run 1.98.0 cargo test --release -p rustui --lib --no-run --locked --offline
    env RUSTC="$(rustup which --toolchain 1.98.0 rustc)" RUSTDOC="$(rustup which --toolchain 1.98.0 rustdoc)" CARGO_TARGET_DIR="$HOME/.cache/rustui-target-f0-10-gpui-fast" rustup run 1.98.0 cargo test -p rustui --lib --no-run --locked --offline
    env RUSTC="$(rustup which --toolchain 1.98.0 rustc)" RUSTDOC="$(rustup which --toolchain 1.98.0 rustdoc)" CARGO_TARGET_DIR="$HOME/.cache/rustui-target-f0-10-gpui-fast" rustup run 1.98.0 cargo test --release -p rustui --lib --no-run --locked --offline

All four builds exited 0. Timings use only release binaries: Cargo release profile, default opt-level=3, debug-assertions=false, repository lto="thin", codegen-units=1. The debug/test-profile binaries were built to identify and compare the historical profile discrepancy, not used for official measurements. Each timed invocation was:

    env RUSTC="$(rustup which --toolchain 1.98.0 rustc)" RUSTDOC="$(rustup which --toolchain 1.98.0 rustdoc)" CARGO_TARGET_DIR="$BACKEND_TARGET" rustup run 1.98.0 cargo test --release -p rustui --lib "$FILTER" --locked --offline -- --ignored --nocapture

Filters, each run five times per backend: f0_long_list_scroll_benchmark, f0_large_editor_typing_benchmark, f0_idle_redraw_invalidation_benchmark, and owner_update_and_draw_workload. For each workload, runs were interleaved gpui-pre, Fast five times. No timed log contained a Cargo Compiling line; all 40 test invocations passed.

These are warm-window measurements, not cold-launch measurements. The scroll, editor, and idle tests create their view and draw once before starting timers. Scrolling then includes new rows exposed by each changing scroll position; editor timing starts after initial document layout and a simulated click; idle measures unchanged draws on the retained view. The 1,000-row workload draws once, then discards sample 0 as warm-up in each update/forced-draw mode and records samples 1–10; this report uses its 1,000-row forced_draw samples. For those ten samples per invocation, median is the upper middle sample and p95 is the maximum (nearest-rank definition used by the test harness). For all workloads, aggregate median and p95 are the medians of the five per-invocation medians and p95 values. Spread is (max(run median) - min(run median)) / aggregate median.

## F0-10 interleaved results

Per-invocation values are median / p95 milliseconds. Aggregate columns report median of the five medians / median of the five p95s. Every spread was below 10%, so no workload needed a repeat set.

| Workload | gpui-pre runs 1–5 (median / p95 ms) | gpui-pre aggregate; spread | Fast runs 1–5 (median / p95 ms) | Fast aggregate; spread |
|---|---|---:|---|---:|
| 5,000-row scrolling, 300 frames | 1.274208 / 1.495292; 1.265375 / 1.512959; 1.297167 / 1.511708; 1.286083 / 1.513542; 1.260500 / 1.556208 | 1.274208 / 1.512959 ms; 2.88% | 0.626625 / 0.812625; 0.622292 / 0.751750; 0.626125 / 0.775792; 0.619791 / 0.774583; 0.615541 / 0.792000 | 0.622292 / 0.775792 ms; 1.78% |
| 10,000-line Rust editor, 100 edits | 0.646250 / 0.979959; 0.654459 / 0.905541; 0.678584 / 0.955334; 0.655584 / 0.984166; 0.685458 / 1.027250 | 0.655584 / 0.979959 ms; 5.98% | 0.659959 / 0.956083; 0.629792 / 1.021792; 0.631708 / 0.976791; 0.633291 / 0.957000; 0.637250 / 0.916000 | 0.633291 / 0.957000 ms; 4.76% |
| Idle draw, 60 mounted panels, 60 frames | 0.266167 / 0.332209; 0.255459 / 0.320167; 0.262667 / 0.343584; 0.257917 / 0.351917; 0.257500 / 0.333958 | 0.257917 / 0.333958 ms; 4.15% | 0.016458 / 0.017500; 0.015834 / 0.016292; 0.015792 / 0.016041; 0.016500 / 0.018375; 0.016458 / 0.026083 | 0.016458 / 0.017500 ms; 4.30% |
| 1,000-row owner update + forced draw | 8.699917 / 9.415304; 8.773129 / 9.033858; 8.762925 / 8.999104; 8.743671 / 9.075250; 8.793417 / 9.004500 | 8.762925 / 9.033858 ms; 1.07% | 5.854621 / 6.660504; 5.663125 / 5.750096; 5.643271 / 6.097308; 5.742750 / 6.128204; 5.494846 / 5.761454 | 5.663125 / 6.097308 ms; 6.35% |

The 1,000-row values are parsed from the emitted ns_per_update values and divided by 1,000,000 to report milliseconds. Each invocation emits ten measured samples per mode after the unreported sample-0 warm-up.

## Historical method comparison and discrepancy

F0-1's committed reproduction command for the three new workloads omitted --release, so it used Cargo's test profile (opt-level=0, debug assertions on); its separate 1,000-row script explicitly used --release. F0-9's report says its replacement runs did not overlap a Cargo build and gives three-run data, but does not preserve the exact command or profile. F0-8 likewise records three-run results but not the command, profile, or target directory. Therefore the historical F0-8/F0-9 command lines and target directories are NOT recoverable from the committed records. The F0-9 values (5.586 ms scroll, 5.068 ms typing, 0.183 ms idle) track the unoptimized scale, while F0-8's 0.630 / 0.699 / 0.017 ms values track the optimized scale. F0-10 confirms the cause: on the same Fast commit and identical test source, release produces 0.622 / 0.633 / 0.016 ms; the F0-1 debug baseline had been 11.277 / 5.775 / 3.702 ms. The discrepancy was a debug test profile in the F0-9 rerun compared with an optimized F0-8 run, not a Fast rendering regression. F0-10 uses release for both backends with identical warm-up and workload code.

Historical exact build commands, opt-level/assertion settings, and target paths are unavailable for F0-8 and F0-9. The profile attribution is supported by F0-1's recorded command and F0-10 same-backend release measurements, rather than retained historical build logs.

As a profile diagnostic after the official runs, the same four test filters were each run once per backend using the already-built debug/test binaries, with the identical warm-window setup and no compilation. The three new workloads produced gpui-pre / Fast medians of 11.729 / 5.724 ms (scroll), 5.845 / 4.998 ms (typing), and 3.792 / 0.175 ms (idle), matching the debug-scale F0-1/F0-9 figures. The debug 1,000-row forced-draw medians were 97.113 / 66.149 ms. These one-run diagnostics are not used as official benchmark values. They directly demonstrate that changing only the test profile moves the Fast scroll/typing/idle timings by factors comparable to the F0-8/F0-9 discrepancy.
