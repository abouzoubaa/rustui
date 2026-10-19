# P3 diagnostics-overhead calibration

## Purpose

Measure the overhead of RustUI host-lowering diagnostics on one executable and one fixed workload. Keep the cache mode, leaf count, update schedule, environment, and collector protocol identical between diagnostic-off and diagnostic-on runs. The calibration is not frame, GPU, presentation, or input-latency evidence.

## Probe support

`crates/rustui/examples/cache_comparison.rs` accepts `diagnostics_enabled` in each run configuration. Missing values preserve the existing enabled behavior. The workload artifact records the setting and, when enabled, a per-component list of raw lowering count, accumulated nanoseconds, and maximum nanoseconds. Disabled runs retain an empty diagnostic list. Use an uncached workload with a fixed leaf count for both arms so the enabled arm exercises the instrumentation consistently.

## Required paired run protocol

Build the changed `cache_comparison` example once with `./scripts/cargo-headstart build -p rustui --example cache_comparison --locked --offline` and use that exact binary for all runs. Create a fresh plan with:

```sh
python3 scripts/compare_cache_workload.py plan \
  --session diagnostics-unique-session \
  --binary target/headstart/debug/examples/cache_comparison \
  --workload 32-leaf --diagnostics-overhead
```

The plan holds cache mode at `uncached`, pairs `diagnostics_on` and `diagnostics_off`, and alternates pair order. It contains one warmup pair and five measured pairs. Follow its `launch_argv_for_cua` entries in index order. For each ready process, attach with the documented command in `P3D_CACHE_COMPARISON.md` using the diagnostics plan path and that run's index and PID. Summarize after all twelve captures with `python3 scripts/compare_cache_workload.py summarize --plan <plan-path>`. Each pair differs only in `diagnostics_enabled`. Preserve every run, including failed or excluded captures.

For every run, retain the binary/source/config identities, workload record, raw per-type diagnostic counts and durations, raw process samples, collector sampling durations, attachment CPU/elapsed deltas, sampled RSS, and AX/drag guards. Summarize paired diagnostic-on/off ratios and their spread. Report diagnostics timing only as time spent in `Component::into_element`; nested host totals can overlap and must not be summed as process duration.

## Interpretation

A valid calibration requires complete correctness records, matching workload protocol and binary hashes, unique native process identities, no active accessibility or drag observations, all five measured pairs, and preserved raw samples. Process CPU includes normal UI work, diagnostics, and collection; the paired delta estimates the instrumentation's effect for this specific workload. It does not identify a universal overhead or justify cache promotion. Small or noisy paired differences remain inconclusive.

The earlier P3A record `target/headstart/milestone20-host-diagnostics-gui.json` contains functional observations only and is not this calibration. The P3D cache-comparison sessions always enabled diagnostics and compare cache modes, so they cannot serve as diagnostics-on/off pairs.
