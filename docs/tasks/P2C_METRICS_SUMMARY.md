# P2c: External runtime metrics summary

`scripts/summarize_runtime_metrics.py` validates explicit
`rustui_process_metrics` schema version 1 JSON records and reports descriptive
per-run CPU, RSS, and sampler overhead values. It does not launch a process,
collect samples, invoke Cargo, or depend on a GUI runtime.

```sh
python3 scripts/summarize_runtime_metrics.py target/headstart/performance/raw-record.json
python3 scripts/summarize_runtime_metrics.py target/headstart/performance/run-a.json \
  target/headstart/performance/run-b.json \
  --output target/headstart/performance/summary.json
```

Without `--output`, the report is written to stdout. A requested file uses the
shared exclusive writer and must be directly under
`target/headstart/performance/`; existing outputs and symlink destinations are
rejected. Input records are read-only. Invalid schema or sample data produces a
clear command-line error and nonzero exit. Collector records with no samples,
including an incomplete record where the first sample failed, are rejected with
an explicit error because no sample summary can be computed; they are never
presented as complete results.

Raw monotonic timestamps, CPU nanoseconds, RSS bytes, and sampler duration must
be nonnegative JSON integers. Booleans and non-finite JSON constants are invalid.
CPU percentage is `(last cumulative CPU time - first cumulative CPU time) /`
`observed sample elapsed duration * 100`. It can exceed 100% when a process
uses multiple CPUs. RSS is summarized in bytes. Sampling overhead is
`ps_duration_ns` converted to milliseconds; median and p95 use nearest-rank
percentiles (`ceil(p*n)`, one-based rank). Even-sized medians are the arithmetic
mean of the two middle values. Observed elapsed duration is the first-to-last
sample timestamp span; it is not workload wall time, a W1 cycle measurement, or
frame time.

Each record remains a separate run, including incomplete runs. Unlike workloads
are never combined. The report makes no acceptance verdict, baseline/candidate
comparison, or improvement claim. Frame time, cycle time, and main-thread time
remain unavailable; process CPU and RSS cannot establish FPS or GUI behavior.

Unit coverage is in `scripts/tests/test_summarize_runtime_metrics.py`. No native
runtime measurement, benchmark, GUI run, or FPS check is part of this summary
utility.
