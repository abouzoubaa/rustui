# P3D: paired retained-subtree comparison

## Scope

This milestone adds a controlled comparison of the existing official GPUI
retained-subtree cache. It does not add a renderer, adopt the GPUI Fast fork,
change dependency features/revisions, install patches, or start advanced GPU work.
No performance benefit is claimed before measurements and review.

The original cache benchmark places cached and uncached children in one process.
Its calibration establishes repeatability but cannot attribute process cost to
caching. This example instead uses one binary with two modes and two fixed
workloads: `8-leaf` preserves the original eight-child baseline, while
`32-leaf` varies retained child count. Both use the same root and leaf types,
text/data, four columns, twelve Text views per leaf, 210 × 220 logical-pixel
leaf geometry, ten warmup updates, one hundred measured parent updates, and
one mutation per leaf. The sole rendering difference is applying
`.cached_fixed(210.0, 220.0)`. The selected leaf count is included in each
immutable run configuration and checked against reported values, mounts, and
lowering counts.

The native example is framework implementation code and may use GPUI directly;
the canonical Playground remains a consumer of the public RustUI API.

## Native launch and attach protocol

Build `cache_comparison` separately through the approved Headstart launcher.
The comparison script never builds, launches GUI applications, sends signals,
fetches sources, or applies/promotes patches. Every native run must be launched
through CUA. CLI examples below invoke only the planning/attach/reporting tool.

1. Create a fresh plan against an existing binary. The default is `8-leaf`;
   pass `--workload 32-leaf` for the varied retained-tree case:

   ```sh
   python3 scripts/compare_cache_workload.py plan \
     --session p3d-unique-session \
     --binary target/headstart/debug/examples/cache_comparison \
     --workload 32-leaf
   ```

   This writes an exclusive immutable plan and twelve configuration JSON files
   directly under `target/headstart/performance/`. The plan captures the selected
   workload, binary, relevant RustUI/collector sources, Cargo inputs, launcher,
   and current compiler identities. These are plan-time facts, not proof of which
   source produced the executable; preserve separate matching build provenance
   and the raw build log. Use a fresh session ID for each workload and do not mix
   captures from different workloads in one summary.

2. Follow `runs` in the plan in exact index order. There is one fresh-process
   warmup pair, followed by five measured fresh-process pairs. Pair start order
   alternates cached/uncached, then uncached/cached, reducing simple order bias.
   The two warmup processes are correctness checked but omitted from summaries.

3. Launch each process through CUA with the plan's `launch_argv_for_cua` if the
   native launch surface supports arguments. Otherwise package a byte-identical
   copy of the planned binary as a native app and copy that run's config into:

   ```text
   <app>.app/Contents/MacOS/cache_comparison
   <app>.app/Contents/Resources/cache-comparison.json
   ```

   CUA then opens the bundle. The example reads this resource when launched
   without arguments; an explicit `--config <path>` takes precedence. Config
   contains `run_id`, `mode`, and the planned `leaves` count. Output is fixed
   inside this repository.
   Use fresh IDs and preserve every prior run artifact. Do not launch through a
   shell `open`, shell executable invocation, or Python `Popen`.

4. The example writes `cache-comparison-<run_id>-ready.json`, recording its PID,
   executable path and mode. Attach to that existing PID:

   ```sh
   python3 scripts/compare_cache_workload.py collect \
     --plan target/headstart/performance/cache-comparison-p3d-unique-session-plan.json \
     --run-index 0 --pid <existing-native-PID>
   ```

   The collector verifies binary SHA-256 and `ps` executable identity, requires
   earlier captures to have completed, rejects PID reuse, then reuses
   `collect_runtime_metrics.collect` for raw `ps` samples. After its first sample
   it writes a run/PID-specific start gate. The example waits for this gate before
   warmup, so startup is outside the attached measurement. The collector takes
   another sample after observing the completed result, saves exclusive raw and
   derived evidence, and writes an acknowledgement. The example then exits.

5. Repeat the CUA-launch/attach sequence for all twelve indices, then run:

   ```sh
   python3 scripts/compare_cache_workload.py summarize \
     --plan target/headstart/performance/cache-comparison-p3d-unique-session-plan.json
   ```

   Summarization rechecks plan/source/binary fingerprints, every raw workload
   artifact, expected counter/value/mount invariants, fresh process identities,
   and raw-versus-derived metrics. It emits per-mode median/min/max/range,
   per-pair cached/control ratios, and ratio spread. A zero control denominator
   yields unavailable ratios. No automatic pass/speedup/promotion decision exists.

## Correctness and measurement limits

Each completed run must show exactly 100 parent host lowerings; zero cached or
800 uncached child host lowerings during that phase; eight further leaf
lowerings after local mutation; eight rendered values of one; and one mount per
leaf. The raw JSON retains these counters and logical values. This checks reuse
and child invalidation; it does not establish correct pixels or native input.

Any observed active accessibility session or native drag excludes the run.
Typed environment scope is empty by construction. The existing framework cache
fallbacks are preserved. Some CUA connections activate accessibility; preserve
that excluded result and report the environment limit instead of treating it as
a cache comparison. Sustained no-AX native-drag acceptance remains a separate
open probe and is not settled here.

Launching a packaged run through CUA-controlled Finder is permitted when directly
selecting the benchmark app would activate AX. Do not inspect or select the
benchmark's accessibility tree merely to obtain its PID: read the ready artifact
and attach the process collector. The example itself samples GPUI's actual
`Window::is_a11y_active()` on every active callback; any observed activation
excludes the run regardless of launch method. There is no synthetic no-AX flag,
forced eligibility override, or permission to relabel an excluded result. If the
available CUA surface cannot launch without AX activation, preserve the exclusion
and report that native cache measurement remains blocked by that environment.

The reported CPU delta spans collector attachment through the final sample.
It includes warmup, measured updates, local mutation, diagnostics and the short
post-result collection wait; it excludes process startup and teardown. RSS is
the median and maximum of sampled resident memory, not OS peak RSS. Elapsed
time is the attachment interval, not GPUI frame, presentation, GPU, main-thread,
or input-to-photon timing. Sampling and process CPU counters have finite
resolution. Small/noisy results must remain inconclusive. Keep the same display,
appearance, machine/power settings and unrelated workload across the session.

The process collector has a 45-second bound and a 100-ms nominal interval. It
keeps partial samples on sampling failures; missing result/incorrect/excluded
runs fail the comparison. The example has a separate 120-second timer and quits
on acknowledgement or timeout. The collector never kills a process; a stuck
native process requires CUA/user handling. Failures and partial session artifacts
are retained; start a new session instead of overwriting evidence.

## Verification status

Implementation files are the native example, attach-only comparison script,
focused fixture tests, and this task contract. Tests mock native process sampling
and verify the fixed alternating schedule, provenance/config drift, workload
invariants, sample validation, start/ack ordering, excluded/partial retention,
paired summaries, warmup exclusion, changed artifacts/counters/PIDs, and zero
denominators. They launch no GUI applications.

The initial focused Python suite passed **21 tests, 0 failed**; the log is `target/headstart/p3d-final-python-tests.log`. Its final guard tests exclude runs if accessibility or native drag is observed, including late observations. The comparison tool, example build, and corrected locked/offline all-targets check passed at that milestone; see `docs/BUILDING.md` for build evidence. The completed Finder/CUA-only session `p3d-review-01` is summarized in `target/headstart/performance/cache-comparison-p3d-review-01-summary.json` and bound to its immutable plan at `target/headstart/performance/cache-comparison-p3d-review-01-plan.json`.

After adding workload selection, `python3 -m unittest scripts/tests/test_compare_cache_workload.py` passed **24 tests, 0 failed**. The fixtures cover both workload protocols and their leaf-count invariants; they do not build or launch the native example.

## Native benchmark results (2026-10-18)

All 12 planned processes completed with correctness checks true and no observed accessibility activation or native drag. This was one correctness-checked warmup pair, excluded from timing summaries, plus five measured cached/uncached pairs. Each run used one opt-level-0 workload: 8 leaves, 12 text lines per leaf, 10 warmup updates, 100 measured parent updates, then one mutation per leaf. Across all runs, cached parent updates produced 0 child lowerings versus 800 in uncached mode; each mode produced 8 post-mutation child lowerings and 8 expected rendered values. These native results cover only the `8-leaf` workload. The `32-leaf` selection has no native measurements yet and requires its own build provenance, CUA-launched session, and summary.

The median cached/uncached attached CPU ratio was **0.2895** (about 71% lower CPU time for the cached mode in this workload); paired ratios ranged from **0.2848 to 0.3154**. This CPU interval includes warmup, measured updates, mutation, diagnostics, and post-result collection wait; it is not frame time. Median max-sampled RSS ratio was **0.9956** (about 0.4% lower), with a paired range of 0.9924–1.0028. Sampled RSS is not OS peak memory. These measurements include instrumentation and support only this single workload and environment; they do not establish frame, GPU, presentation, input-to-photon, or generalized application speedup. No acceptance threshold or automatic speedup verdict was set.

Further work should add calibrated workloads with varied retained-tree sizes/content and repeat under controlled machine conditions. Native pixel, accessibility, and drag acceptance remain separate; these benchmark runs were launched through Finder without benchmark accessibility inspection and only establish that AX/drag were not observed during their callbacks. The protocol and results do not constitute visual acceptance.
