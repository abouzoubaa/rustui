# P3 diagnostics-overhead calibration results

This record reports the calibration defined by
`docs/tasks/P3_DIAGNOSTICS_OVERHEAD_CALIBRATION.md`. That protocol file is
also a hashed plan input, so it remains unchanged after planning. This result
record and the roadmap closeout are separate from the frozen experiment inputs.

## Build and provenance

The `cache_comparison` example was rebuilt from HEAD
`515a0e389e2d18a05874af98dd416fbd4ba0399c` with the locked, offline Headstart
command; it exited 0. The raw build log is
`target/headstart/evidence/diagnostics-build-515a0e.log`. The provenance record
is `target/headstart/evidence/diagnostics-build-provenance-515a0e.json`; it
binds executable SHA-256
`463925f1d80a26923db35da3247e813042adcf87607bb42c624bc5b88e63c527`,
compiler identity, revision, source snapshot, and workspace inputs. As defined
by the tool, the command and exit status are caller-declared metadata; the
record does not independently certify historical compiler execution.

## Capture acceptance

The immutable plan
`target/headstart/performance/cache-comparison-diag-overhead-515a0e-plan.json`
completed 12 Finder/CUA native captures: one correctness-only warmup pair and
five measured pairs. All used the uncached 32-leaf workload, 12 text lines per
leaf, 10 warmup updates, 100 parent updates, and one mutation per leaf. The
plan summary validator completed successfully at
`target/headstart/performance/cache-comparison-diag-overhead-515a0e-summary.json`.

All 12 PIDs were unique. Every run passed correctness with 32 values of one
and one mount per leaf, and reported `ax_active_seen=false`,
`native_drag_seen=false`, and an empty environment scope. Diagnostics-on
records had 100 parent root lowerings, 3,200 measured leaf lowerings, 3,232
final leaf lowerings, and a mutation delta of 32. Diagnostics-off records
had zero instrumentation counters and no timing records while preserving the
same values, mounts, and correctness. Raw ready, result, final, metrics, and
process-sample artifacts are retained for each run under the plan's session
prefix.

## Results and limits

Across the five measured pairs, the median diagnostics-on/off attached CPU
ratio was **0.995855** (range **0.982688–1.002083**, 1.95% of the median).
Median attached CPU was 9.62 s on and 9.68 s off. Since the paired range
crosses 1.0, this small difference is inconclusive. The median paired
attached-elapsed ratio was **0.990446** (range **0.989967–1.000772**). The
median paired max-sampled-RSS ratio was **1.000000** (range
**0.994326–1.002659**); the median-sampled-RSS ratio was **0.999155** (range
**0.994317–1.000242**).

For diagnostics-on `Component::into_element` timings, the per-run median
`ComparisonLeaf` total was **4,460,388,195 ns** over 3,232 calls; the
`ComparisonRoot` total was **31,229,583 ns** over 101 calls. Nested host totals
can overlap and are not process duration.

Attached process CPU includes ordinary workload and collector activity; the
attachment interval excludes startup and teardown. RSS is sampled rather than
OS peak. This is one dev-profile workload and establishes no universal
diagnostics overhead, frame/GPU/presentation/input latency, visual correctness,
or cache-speed result. No general performance win is claimed.
