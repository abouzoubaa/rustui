# PX-3 — Rust empty-injection fast path

The GPUI Kit candidate is off by default. `RUSTUI_HL_FASTPATH=1` enables reuse of an empty injection-discovery result for the exact pinned Rust grammar and literal injection query. This does not disable macro highlighting or change frame scheduling.

Reuse requires a prior full query scan with zero actual matches, one known incremental edit, no structural changed ranges, no old/new syntax errors, empty injection layers, the original query object, the pinned grammar, and an unchanged language-registry generation. Matches are counted before language resolution, filtering, parse caps or failures; an empty layer list alone is not negative evidence. Unknown/batched edits, stale trees, background results, grammar/query changes and registry registration take the existing full-scan path. Files with macro candidates continue to scan.

The patch adds `rust_empty.rs` and its oracle tests; existing highlighter code gets the eligibility check and one observed-scan result. It is a hashed, reversible proposed patch, applied by script to vendor and owned Kit sources and listed after the approved Fast stack. No approved patch is promoted.

## Verification and measurement

The focused runner uses a disposable copy of the excluded owned Kit package. It keeps the real highlighter sources byte-identical and excludes unrelated upstream unit modules whose external workspace fixtures are absent from the published package. The oracle compares ordered capture names/ranges, final styles and injection layers with a freshly built highlighter. It asserts reuse hits for comment/identifier edits, and covers macro creation/removal, nested token trees, macro rules, delimiters, newlines, undo, query/grammar changes, filtered query matches, unknown/batched edits, stale trees, background results and registry invalidation.

Measurement uses one pinned Rust 1.98.0 release build, damage off, buffered logging and five matched typing pairs in off→on, on→off, off→on, on→off, off→on order; 60 seconds after normal warmup. Both arms keep `RUSTUI_FRAME_PROFILE=1 RUSTUI_G0_LATENCY=1 RUSTUI_LATENCY_BREAKDOWN=0`. Only `RUSTUI_HL_FASTPATH=0/1` changes. The deciding metric is measured-window getrusage user+system CPU seconds per attempted present, with the exact [Stage P verdict rule](MEASUREMENT_RULES.md). No compilation, profiling or other test load overlapped timing. The unrelated indexer and UI Studio processes were paused for all ten captures and resumed afterward; ordinary macOS services remained active. The runtime code was frozen at `5a552f8017173e3441a0ef086ec3378f84e7cb75` on base `973fcbe6ef280e98a6c3ab48b6005d7b3dbc2a6f`; later ticket changes document results only. Binary SHA-256 was identical before and after all captures.

## Matched typing results

The five pairs completed on 2026-11-21, 60 seconds each after the normal two-second warmup, at 1200 × 800 points / scale 2. Each capture handled 600 scripted typing inputs. Unrounded process-CPU changes determine the verdict. No other scene is opted into this Rust-specific candidate.

| Scene | Off median process CPU ms/attempt | On median process CPU ms/attempt | Five paired changes % | Median % | Min % | Max % | P verdict |
|---|---:|---:|---|---:|---:|---:|---|
| typing | 13.838304 | 6.589739 | -54.420, -50.391, -51.601, -53.489, -52.380 | -52.380 | -54.420 | -50.391 | **GAIN** |

Screen-delay values below are pooled from the actual matched input versions in all five captures per arm, using the existing presentedTime correlator with per-process window IDs namespaced. They are not averages of percentiles. CPU-phase and GPU medians are medians of the five capture means. These diagnostics do not change the P verdict.

| Diagnostic | Off | On |
|---|---:|---:|
| Draw + present elapsed CPU ms/attempt | 0.846430 | 1.697465 |
| GPU ms/present | 0.642733 | 0.636708 |
| Actual key-to-screen median ms | 56.823 | 50.637 |
| Actual key-to-screen p95 ms | 67.632 | 58.141 |
| Actual key-to-screen maximum ms | 70.771 | 70.779 |
| Draw + present CPU frames >8.3 ms (all captures) | 0 | 1 |
| Valid displayed input versions / handled | 2596/3000 | 2807/3000 |
| Painted without valid presentation | 404 | 193 |
| Coalesced before paint | 0 | 0 |

Process CPU fell under the specified P rule, while the draw+present elapsed phase increased; the latter is reported separately and is not substituted for process CPU. Key-to-screen delay remains tens of milliseconds. This candidate does not change presentation policy or establish a solution to PX-4's post-GPU wait. Slow-frame counts above are CPU-phase durations, not input-delay counts or presentation gaps. All damage modes were Direct and all non-Partial reasons were `disabled`.

### Individual captures

User/system CPU seconds are measured-window getrusage deltas, excluding warmup, cleanup and buffered exit writes. Delay statistics exclude versions with no valid displayed drawable; their counts remain explicit.

| Pair | Arm | User s | Sys s | CPU s | Attempts | CPU s/attempt | Draw+present ms/attempt | CPU frames >8.3 ms | Shown/inputs | Key-to-screen median / p95 / max ms |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|---|
| 1 | off | 6.540725 | 0.548403 | 7.089128 | 520 | 0.013632938 | 0.846430 | 0 | 520/600 | 56.808 / 65.946 / 70.718 |
| 1 | on | 2.838543 | 0.653651 | 3.492194 | 562 | 0.006213868 | 1.566482 | 0 | 562/600 | 51.131 / 58.107 / 70.599 |
| 2 | off | 6.546157 | 0.550242 | 7.096399 | 522 | 0.013594634 | 0.864702 | 0 | 522/600 | 55.358 / 67.193 / 70.755 |
| 2 | on | 3.068481 | 0.728467 | 3.796948 | 563 | 0.006744135 | 1.819350 | 0 | 563/600 | 50.689 / 58.142 / 64.787 |
| 3 | off | 6.691633 | 0.556070 | 7.247703 | 520 | 0.013937890 | 0.852712 | 0 | 520/600 | 57.552 / 66.229 / 70.763 |
| 3 | on | 3.069981 | 0.714419 | 3.784400 | 561 | 0.006745811 | 1.714945 | 0 | 561/600 | 50.974 / 58.108 / 70.668 |
| 4 | off | 6.650848 | 0.540497 | 7.191345 | 517 | 0.013909758 | 0.820961 | 0 | 517/600 | 57.565 / 67.414 / 70.771 |
| 4 | on | 2.940966 | 0.688440 | 3.629406 | 561 | 0.006469529 | 1.656321 | 1 | 561/600 | 50.435 / 58.123 / 67.139 |
| 5 | off | 6.643284 | 0.511119 | 7.154403 | 517 | 0.013838304 | 0.794124 | 0 | 517/600 | 56.542 / 67.867 / 70.757 |
| 5 | on | 2.983459 | 0.706795 | 3.690254 | 560 | 0.006589739 | 1.697465 | 0 | 560/600 | 51.341 / 58.185 / 70.779 |

## Validation and disposition

Eight real highlighter oracle tests pass, including asserted reuse hits and comparison against fresh highlighting. The owner-authorized Kit verifier extension has eleven focused tests covering approved anchors, tampered patches/files/checksums, unsafe paths, symlinks and exact new-file absence/reapplication. The cumulative active candidate stack proves exact reverse/apply restoration; owned sources match vendor sources. No approved hash table, release script or CI configuration was edited.

`scripts/check.sh` passed all seven groups: 1,874 total tests, 1,867 passed and 7 ignored. `./scripts/verify-release.sh source` passed 695 tests. The pinned release build passed. Final source CI and all review comments must pass/be addressed before merge. The candidate remains off by default and proposed; no approved-patch promotion is made. P1, latency behavior fixes, G2 telemetry and G2-8b are outside this ticket.
