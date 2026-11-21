# PX-2 diagnostics on the draft G2 branch

The branch-specific `gpui-fast-presented-latency-v1-fast` candidate is rebased after the existing G2 E/F patches. Its generated sources preserve the existing instance-counting Skip hook and add a `backoff` reason. No render choice or F threshold changes. Base: `1bf1158e9943737ecbcbbea212f3ce747c8c3db7`. PR #163 remains draft.

The isolated checkout release build passed, and `./scripts/verify-release.sh source` passed 688 tests. `scripts/check.sh` passed 1,874 total tests (1,867 passed, 7 ignored) in that checkout, before the prime-label correction described below. Those full checks have NOT VERIFIED the final corrected label. Results from the integration branch must not be substituted for F's back-off sequence.

The older G2 `damage_cpu` and instance recorders recognize only `RUSTUI_FRAME_LOG=1`. Consequently their counters with `buffered` are not measured; zero must not be inferred. PX-2 reason records and actual-present tags support buffered logging independently. Integration of the benchmark parser with those legacy counters remains unfinished; no counter or frame-choice policy is silently changed here.

## Fresh F-on reason captures

Five 60-second windows after normal warmup, one release build, `RUSTUI_DAMAGE=1 RUSTUI_DAMAGE_BACKOFF=1 RUSTUI_FRAME_LOG=buffered RUSTUI_FRAME_PROFILE=1 RUSTUI_G0_LATENCY=1`. Builds and local checks finished before recording; no compilation or profiling overlapped these captures. The unrelated indexer was paused. These are reason diagnostics, not matched CPU-gate measurements.

| Scene | Skip | Copy | Partial | Retained | Direct | Raw non-Partial reasons |
|---|---:|---:|---:|---:|---:|---|
| idle | 0 | 0 | 114 | 0 | 0 | none |
| typing | 80 | 0 | 439 | 0 | 0 | identical: 80 |
| hover | 0 | 0 | 0 | 43 | 7153 | large: 566, backoff: 6630 |
| log | 0 | 0 | 0 | 0 | 575 | large: 24, backoff: 551 |
| drag | 1 | 1111 | 6073 | 0 | 0 | identical: 1, required: 1111 |

The captured hover sequence exposed a diagnostic-label error: its 43 Retained priming frames were included in the raw `large` count (566), rather than labeled `prime`. The candidate now labels Retained frames on the E/F branch `prime`; render selection is unchanged. The corrected label passes hash/reversal and generated-source checks, but its release rebuild, full checks and fresh reason retest are **NOT VERIFIED**. The table deliberately preserves the pre-correction records. No Stage G gate is declared complete.

PX-2 source CI passed and PR #166 merged into g-integration. The G2-specific diagnostic import remains WIP on draft PR #163, including the label retest and legacy buffered-counter/parser compatibility noted above. PX-3 has not started. The two-hour PX-2 timebox requires a stop with WIP pushed.
