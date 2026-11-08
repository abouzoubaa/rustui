# C3 — 1,000-row redraw profile

The release test-window workload was measured on this Mac with `scripts/perf-baseline.sh`; each result is the script's median across three runs. The baseline was 11.8882 ms per update with forced draw. The final run with thin LTO measured 8.0305 ms (4.00% run spread), a 3.8577 ms reduction. The result remains above the 8.0 ms target. This workload is not a native frame or GPU benchmark.

## Sampled breakdown

`sample` was attached during the 1,000-row forced-draw portion of the release test (1,029 worker samples before, 682 after). The table multiplies each sample share by the independently measured median; these are estimates, not direct phase timings. The after sample and its 8.4849 ms median precede the owner-directed shared cache, per-pass reuse, and thin-LTO changes; a final phase sample was not taken. Stack ancestry classifies layout and prepaint/paint. Samples do not distinguish owner `body()` from RustUI element building, or reconcile/diff from other host work; those are shown together rather than presented as separate measurements.

| Phase | Before ms | Before % | After ms | After % |
|---|---:|---:|---:|---:|
| Owner body and RustUI view building/lowering | 3.789 | 31.9 | 0.958 | 11.3 |
| RustUI reconcile/diff | Not isolated | — | Not isolated | — |
| GPUI layout | 2.415 | 20.3 | 1.879 | 22.1 |
| GPUI prepaint + paint | 5.199 | 43.7 | 5.200 | 61.3 |
| Other, including reconcile/diff | 0.485 | 4.1 | 0.448 | 5.3 |

GPUI layout plus prepaint/paint was estimated at 7.079 ms for the intermediate after sample; sampling alone does not prove an exact phase duration or establish that those phases exceed 8 ms. Separately, RustUI's existing host diagnostic recorded 100.817 ms over 110 updates and 220 lowerings, or approximately 0.9165 ms of combined lowering per update. That diagnostic run predates the final font-family move.

## Baseline RustUI hotspots

The following estimates attribute worker samples to the nearest RustUI frame, excluding the workload driver. They are not exact instrumentation and may include work called by each function.

| RustUI function | Samples | Estimated ms/update | Share |
|---|---:|---:|---:|
| `resolve_component_style_from_record` | 122 | 1.409 | 11.86% |
| `TextView::into_element` | 90 | 1.040 | 8.75% |
| Drop `ResolvedComponentStyle` | 26 | 0.300 | 2.53% |
| `selector_for_mode` | 17 | 0.196 | 1.65% |
| `Workload::into_element` | 7 | 0.081 | 0.68% |

## Experiments

| Change | Forced-draw median ms | Change from preceding run |
|---|---:|---:|
| Baseline | 11.8882 | — |
| Cache resolved styles (broader intermediate cache) | 10.5255 | -1.3627 |
| Avoid cloning full style maps for text | 9.2995 | -1.2260 |
| Reuse four immutable text selectors | 9.1370 | -0.1625 |
| Cache text metrics and shared font family | 8.6953 | -0.4417 |
| Replace broader cache with text-only cache | 8.5895 | -0.1058 |
| Move cached font family into element | 8.4849 | -0.1046 |
| Shared `Rc` style-resolution cache | 8.5608 | +0.0759 |
| Per-pass text selector and part reuse | 8.6900 | +0.1292 |
| Thin LTO and one codegen unit | 8.1660 | -0.5240 |
| Final cache invalidation and coverage-path correction | 8.0305 | -0.1355 |

These sequential differences include normal run-to-run variation and are not controlled causal estimates. Experiments with contiguous text-child storage, shared parent text styles, direct style assignment, and an indexed cache did not show a clear benefit and were reverted. The final code retains both the shared style cache and the text-specific cache; theme reactivation and appearance changes clear them. `ActiveTheme::component_style_with` now returns an `Rc` as requested. Cargo release profiles are not inherited by downstream applications.

`perf_baseline.json` records the checkout's parent commit because the measurements were made on uncommitted RustUI changes; this report and its accompanying commit contain the measured changes.
