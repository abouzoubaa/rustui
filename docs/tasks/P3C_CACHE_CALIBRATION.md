# P3C: Cache workload CPU and RSS calibration

`scripts/calibrate_cache_workload.py` repeats the existing
`crates/rustui/examples/cache_benchmark.rs` default workload in fresh processes.
It does not build or modify the example. Build the example separately with the
approved `./scripts/cargo-headstart` launcher, then run from the repository root:

```sh
python3 scripts/calibrate_cache_workload.py \
  --build-manifest target/headstart/evidence/<matching-build-provenance>.json
```

The manifest is optional. If used, supply an existing
`scripts/record_build_provenance.py` record captured for this build. The runner
links its hash and declared build facts but cannot prove the binary was produced
by that build. It records the selected binary's SHA-256, size, the current
Headstart compiler's path, SHA-256 and `-vV` output, machine details, and hashes
of the example source, lockfile, launcher, and calibration script. A changed binary or input hash
defines a different calibration; do not pool those runs.

## Fixed protocol

- One fresh process warmup run, followed by five fresh process measured runs.
- Every process executes the example's 10 warmup and 100 measured parent updates,
  then one cached-child and one uncached-child mutation.
- A run is accepted only when the example reports `complete`, no active
  accessibility session, zero cached and 100 uncached child lowerings during
  parent updates, both child mutations observed, and rendered values of one.
- Each run has a 45-second outer timeout. The example has its own 30-second
  frame-progress timeout. The runner starts an isolated process group, kills
  that group on timeout, and waits up to five more seconds to reap the direct
  child. A run that cannot be reaped in that period is marked failed with
  `reaped: false`; the bounded wait is not a guarantee of OS cleanup. Failed
  or excluded runs fail the calibration and retain available stdout, stderr,
  process usage, and workload JSON in a partial failure record.
- The runner uses macOS `wait4` usage for user CPU, system CPU, and peak resident
  bytes. Wall elapsed time uses a monotonic clock. The JSON retains each run's
  raw workload result, artifact path and hash, stdout, stderr, and measurements.
  Its summary gives median, minimum, maximum, range, and range as a percentage
  of median over the five measured process runs. The warmup process is excluded
  from the summary.

CPU and RSS are **whole-process** values. They include process startup, GPUI
window creation, example warmup, the 100 measured updates, child mutations, and
teardown. They are not CPU or RSS for just the measured frames. Both cached and
uncached children run inside the same process, so these values do not show a
cached-versus-uncached CPU speedup. They establish repeatability and a machine
baseline for this exact mixed workload. A speed comparison needs separately
controlled, equivalent cached-only and uncached-only workloads, with their own
source and native acceptance review before implementation.

The output is a new `target/headstart/cache-calibration-*.json` file. `--output`
must also stay under `target/headstart`; existing files are never replaced. Keep that
record and its per-process `cache-benchmark-*.json` artifacts together for
review. Compiler and source fingerprints capture what was present when the
runner started; they do not replace build logs or native GUI acceptance.

## Executed baseline (M33)

The root ran the built dev example with one excluded process warmup followed by five measured fresh-process repetitions. All six workload records were accepted: AX inactive, 10 warmup parent updates, 100 measured updates, cached child lowerings 0 versus uncached 100, and self mutations produced final values 1/1.

Five measured runs: CPU median 0.452508 s (min 0.433453, max 0.480554; range 10.41% of median), peak RSS median 114868224 bytes (range 622592 bytes), whole-process elapsed median 2.977852208 s. These are development-profile whole-process baselines including startup and teardown, not a cache speedup or GPU/frame timing.

Evidence: `target/headstart/milestone33-cache-calibration.json`, raw per-run files referenced there, `target/headstart/milestone33-cache-calibration.log`, and `target/headstart/evidence/milestone33-cache-calibration-build.json`.

## Update 2026-10-19 — workload scenario matrix

The cache example now exposes `small`, `medium`, and `large` scenarios with 1,
8, and 32 distinct keyed cached children per side, matched by an equal uncached
set. The evidence record includes the scenario and child count; expected
uncached parent-lowering counts scale with children times measured updates. The
calibration runner defaults to all three and can select one scenario. Focused
Python tests, Rust formatting, and the Headstart example build passed. No
calibration run was executed, so this adds workload coverage but no new CPU/RSS
result or performance claim.


## M98 — active theme and calibrated scenario matrix

The first matrix attempt aborted before producing a workload because the example constructed themed components without activating a RustUI theme. The example now loads the bundled `themes/macos27.json` and activates its Light appearance before building the component tree. The Headstart example check and build passed.

The rerun completed all 18 records: one warm-up and five measured fresh processes for each of `small`, `medium`, and `large`; every record passed workload validation. Whole-process medians were:

| Scenario | Children per side | CPU median | CPU range as % of median | Peak RSS median | RSS range as % of median |
| --- | ---: | ---: | ---: | ---: | ---: |
| Small | 1 | 0.462208 s | 11.19% | 124,747,776 B | 0.63% |
| Medium | 8 | 0.665370 s | 4.38% | 125,353,984 B | 0.39% |
| Large | 32 | 1.329313 s | 8.23% | 126,746,624 B | 0.49% |

These figures characterize whole-process CPU, peak RSS, and elapsed time for this mixed cached/uncached workload on this machine. They do not measure cached-versus-uncached speedup, frame latency, GPU work, or general performance. Evidence: `target/headstart/evidence/cache-calibration-scenario-matrix-20261019-fixed.json`. The benchmark process exited; no process remained, and its disposable example executable was removed while preserving shared Cargo caches and workload evidence.
