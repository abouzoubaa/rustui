# W1 GUI acceptance and calibrated CPU/memory baseline

Completed 2026-10-18 for the bounded W1 workload. This records current behavior
and cost before implementing optimizations.

## What was checked

Native screenshot inspection verified reset, independent Alpha/Beta increments,
title prefix on/off, reversed and restored order, and retention of each child's
count and separately entered notes. Reset cleared both counters, notes, titles,
order, and the previous result. The explicit visual fail and pass controls also
updated their displayed result correctly; the intentional fail-control exercise
was not a component correctness failure.

The initial story required scrolling between controls and results. W1 now uses
a compact view with both retained children side by side after Reset W1, keeping
the controls, counters and notes visible at 1320 × 820. The normal pre-W1 story
keeps its original demonstration. Child increment controls now expose their
title and count in accessibility labels, permitting readback after each action.

The measurement driver checked seven states per cycle: reset plus six actions.
Each check compared both child counts, titles and order against the expected
values. **90 measured cycles / 630 state observations passed**, in addition to
nine warm-up cycles. These are mounted accessibility observations; screenshot
inspection separately established visual behavior. Neither confirms display
presentation timestamps.

## Fixed protocol

- Three fresh processes using the identical development-build executable.
- macOS 27, arm64, 10 logical CPUs, AC power; existing user applications preserved.
- Dark macOS 27 theme, English, 1320 × 820 logical-point window, preview at the top.
- Three warm-up cycles, then a 10-second visible-idle capture per launch.
- Thirty measured cycles per launch; each resets and performs the same six actions.
- Process CPU/RSS sampled with `ps` every 250 ms. Collection stops after the
  native driver completes; PID/start identity is guarded throughout collection.
- Native UI automation observes accessibility after every action. Its roughly
  6.8-second cycle duration includes tool/observation delays and is not app latency.

Binary SHA-256:
`2ee96b4ef7a7c5502891b603db36adea82833caac07a809a6697e38fa13fba8a`.
Manifests preserve source hashes, process identity, load averages, power and
available thermal-status output. Report generation rejects mismatched build,
protocol, process identity, incomplete collection, or unexpected child traces.

## Results

| Fresh launch | Cycles | Total process CPU | Median resident memory | Peak resident memory |
| --- | ---: | ---: | ---: | ---: |
| 1 | 30 | 33.64 s | 124.000 MiB | 124.313 MiB |
| 2 | 30 | 34.12 s | 124.000 MiB | 124.578 MiB |
| 3 | 30 | 34.73 s | 124.125 MiB | 124.625 MiB |

Gross CPU cost: median **1.137 CPU seconds/cycle**, with a **3.19%** between-launch
range relative to the median and **1.60%** sample coefficient of variation.
These are CPU seconds spent, not elapsed response time.

Captures include pauses between automation batches, so their durations differ
(264.760, 296.505 and 306.255 seconds). Idle captures used 0.25, 0.25 and 0.26 CPU
seconds over approximately ten seconds each. Subtracting each measured idle rate
across its full active window gives a separately labeled estimate of 0.901,
0.890 and 0.892 CPU seconds/cycle above idle, a **1.17%** observed range. This
estimate assumes steady idle cost; gross observations remain preserved.

Median `ps` call overhead was 6.38–6.43 ms; per-launch p95 was 11.31–13.94 ms.
The CPU clock displayed by `ps` has 10 ms resolution. RSS measures resident
memory, not physical footprint or GPU memory.

This is an initial same-build repeatability calibration, not a confidence
interval, an approved automatic regression threshold, or evidence of speedup.
Future candidates need the same protocol and a contemporary baseline. Small
differences near the observed spread require more repetitions. Release-build,
GPU, main-thread, frame and input-to-display latency require separate measurements.

## Evidence and rerun tools

All evidence is under `target/headstart/performance/`:

- `w1-calibration-session{1,2,3}-manifest.json`: frozen inputs and host conditions.
- `process-w1-calibration-{idle,active}{1,2,3}.json`: raw process samples.
- `w1-calibration-native.json`: native execution/observation summary.
- `w1-calibration-report-final.json`: calculations, limitations and evidence hashes.

`scripts/w1_native_benchmark.js` contains the CUA-only native driver; it is not a
standalone Node app. `scripts/collect_w1_baseline.py` reuses the process collector
and exclusive writer, ending at a caller-created stop file. Use fresh run IDs
and stop filenames for reruns. The report script currently analyzes the recorded
three-session filename convention; use a separate evidence directory or adapt
the input naming explicitly for a new calibration.

```sh
python3 scripts/collect_w1_baseline.py --pid PID --run-id UNIQUE --stop-file UNIQUE_STOP_PATH --binary PATH_TO_EXECUTABLE
python3 scripts/summarize_w1_baseline.py --output target/headstart/performance/UNIQUE_REPORT.json
```

Final build exited 0 (`target/headstart/w1-baseline-build.log`), scoped rustfmt
exited 0, and the metric regression suite passed **18 tests**, including the new
stop-boundary test (`target/headstart/w1-metrics-regression-tests-final.log`).
The native driver passed `node --check`. All six live process captures and final
report generation succeeded. The existing `block 0.1.6` future-compatibility
warning remains. No upstream revisions, vendored sources or optimization patches
changed. The GPUI Fast fork was not adopted.

The canonical app bundle was updated. The separate W1 Acceptance verification
app remains available for inspection; existing user app processes were preserved.

## Release-build recalibration — 2026-10-26

Three fresh **Playground test-app launches**, all controlled from the single
production-consolidation implementation chat, completed three warmups and thirty
measured W1 cycles each. All **90 measured cycles / 630 mounted child-state
observations passed**; nine warmup cycles were excluded from measurements. The
window was independently checked at 2640×1640 screenshot pixels / 1320×820 logical
points, with Dark macOS 27 appearance and English. Existing user applications
were preserved. These test runs are not additional Codex sessions or writers.

The optimized executable SHA-256 was
`15c71ff4d3cef06cd9483f2413d00a1c2a5e1e83d052c2befd74ce6b8aa08f78`.
Source/protocol checkpoint: `4e00ad14`; application sources are unchanged from
`5a37a8d6`. The reporter now checks manifest cycle denominators and integer native
observations, reports the actual build profile, and keeps relative statistics
unavailable for nonpositive denominators. Five focused regressions pass. A rerun
against the old development calibration preserves its session values and
repeatability calculations exactly.

| Fresh test run | Measured cycles | Total process CPU | Median RSS | Peak RSS |
| --- | ---: | ---: | ---: | ---: |
| 1 | 30 | 23.57 s | 125.156 MiB | 125.438 MiB |
| 2 | 30 | 24.23 s | 124.953 MiB | 125.281 MiB |
| 3 | 30 | 24.70 s | 125.875 MiB | 126.203 MiB |

Gross process CPU has a median **0.807667 CPU seconds per cycle**, a **4.664%**
between-run range relative to the median, and **2.349%** sample coefficient of
variation. The separately labeled idle-adjusted estimate has a median 0.685880
CPU seconds per cycle and 2.192% observed range. It assumes stationary idle cost.
Active capture windows lasted 249.505, 269.754 and 346.500 seconds and include
inter-batch pauses. Automation cycle durations are not application response time.
The third run's resident set fell by about 21.5 MiB near the end; its raw samples
and extrema are preserved. This bounded record does not establish leak freedom.

This is a contemporary release CPU/RSS baseline and descriptive repeatability,
not a speedup comparison or an approved automatic regression threshold. Frame,
main-thread, GPU and input-to-display latency, representative collection/editor
workloads, and wider platform performance acceptance remain open. Accessibility
and VoiceOver are undone and deferred; AX text was used only to read functional
child state.

Evidence is under `target/headstart/performance/w1-release-20261026-4e00ad14/`:
three manifests, complete idle/active raw process captures, compact native trace
and per-run summaries, build/notice provenance, `report.json`, independent hash
and RSS verification, and cleanup. The original exclusive report is
`target/headstart/performance/w1-release-20261026-4e00ad14-report.json`.
The reporter requires output directly under `target/headstart/performance/`;
a rejected nested output attempt did not modify measurement records.

All three owned app processes stopped, the disposable bundle and standalone
executable were removed, Cargo caches and evidence were preserved, and the GUI
slot was released. Independent fresh-target compilation and compiler/bootstrap
reproduction are separate release gates.
