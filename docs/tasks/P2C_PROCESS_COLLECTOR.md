# P2C: external process metrics collector

`scripts/collect_runtime_metrics.py` samples an existing PID with `ps`. It never launches, controls, or modifies the process. This is an opt-in measurement aid, not an accepted idle or W1 baseline.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 scripts/collect_runtime_metrics.py \
  --pid 12345 --duration 10 --interval 1 --label local-idle --workload idle
```

Optional `--run-id` selects a safe output name; `--output` selects a direct `.json` file inside `target/headstart/performance/`. Files are created exclusively and never overwritten. `--provenance PATH` records the path and SHA-256 of an existing build provenance file as a caller-declared association. The hash does not prove which binary is running.

`--duration` and `--interval` accept positive finite seconds at nanosecond precision. The interval must not exceed the duration.

The version 1 JSON record contains the requested PID, its `ps lstart` identity, run label and workload, platform context, requested duration and interval in nanoseconds, and samples. Each sample contains `elapsed_ns` (the midpoint of the `ps` call relative to the monotonic start), `ps_duration_ns`, cumulative `cpu_time_ns`, and `rss_bytes` converted from KiB. A successful run has `completion.status = "complete"` and at least two samples. Exit, PID reuse, malformed `ps` output, timeout, or fewer than two samples yields `"incomplete"`, an error, a nonzero exit, and an exclusive partial JSON record when output can be written. The collector checks the same PID and start identity at every sample.

The collector calls `ps -p PID -o pid= -o lstart= -o time= -o rss=` with `LC_ALL=C` and a five-second timeout. CPU time is parsed exactly to integer nanoseconds. Scheduling is anchored to monotonic elapsed time; missed ticks are skipped. A final sample is scheduled at the requested duration when the interval does not land on it.

`ps` CPU time is a cumulative process counter with platform-specific reporting resolution, often coarser than nanoseconds despite the stored unit. `rss` is a sampled resident set size, not peak memory or allocation count. Each `ps` call has overhead and observes the process during an interval; its midpoint is an estimate. A process can do work between samples. These counters cannot establish cycle time, FPS, frame time, main thread time, GUI responsiveness, or workload correctness. The JSON explicitly marks cycle, frame, and main thread timing unavailable. Compare runs only with the workload, process identity, and sampling settings stated alongside them.
