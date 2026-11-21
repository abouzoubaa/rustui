# PX-3b — typing draw/present profile

PX-3b profiles the existing off-by-default PX-3 candidate; no runtime source, candidate patch, switch default, benchmark script or presentation policy changes in this ticket. Base: the current `p-integration` head after P1.

## Method

One pinned Rust 1.98.0 release build through `scripts/pinned-toolchain.sh`, shared target cache, with symbols added by `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_STRIP=none`. Same executable in both arms, with its checksum checked before and after. Samply 0.13.1, 1,000 Hz, all process threads, `record --save-only --unstable-presymbolicate`; typing off first, on second, one capture each. Each app ran `--g0-scene typing --duration 60`, with normal two-second warmup excluded by measurement markers. Both captures validated 600 edits at 1200 × 800 points, scale 2.

Both arms: `RUSTUI_DAMAGE=0 RUSTUI_FRAME_LOG=buffered RUSTUI_FRAME_PROFILE=1 RUSTUI_G0_LATENCY=1 RUSTUI_LATENCY_BREAKDOWN=0`. Only `RUSTUI_HL_FASTPATH=0/1` changed. No compilation, second profiler or heavy analysis overlapped recording. The unrelated indexer and UI Studio were paused across both captures and resumed afterward. Raw profiles, symbols, logs and throwaway analyzers remain ignored local artifacts in `target/headstart/px3b/`.

Resolve canonical physical function symbols with the saved symbol sidecars. Clip per-thread CPU deltas to the measured window, deduplicate recursion for inclusive cost, and divide by timed present attempts. Draw/present attribution requires an actual `Window::draw`/`Window::present` caller in the sample stack; interpolating long idle sample intervals over short phase windows would smear unrelated input/kernel CPU into those phases. Inclusive rows overlap and must not be added. This is one diagnostic profile pair, **not a five-pair P verdict or a latency improvement claim**.

## Results

The historical PX-3 paired report was 0.846430 → 1.697465 ms/attempt for its draw+present elapsed diagnostic. These new profiled captures reproduce the direction, not that exact magnitude:

| Metric | Fast path off | Fast path on | On − off |
|---|---:|---:|---:|
| Attempted presents | 522.000000 | 562.000000 | +40.000000 |
| Draw calls | 599.000000 | 599.000000 | +0.000000 |
| getrusage process CPU ms/attempt | 12.757291 | 5.570347 | -7.186944 |
| Existing benchmark draw+present elapsed ms/attempt | 0.914686 | 1.451792 | +0.537106 |
| All draw+present elapsed ms/attempt | 1.026105 | 1.543513 | +0.517408 |
| Sampled draw+present CPU ms/attempt | 0.774966 | 0.832841 | +0.057875 |
| Elapsed minus sampled CPU ms/attempt | 0.251140 | 0.710672 | +0.459532 |

“All draw+present” sums **every** measured draw duration plus every present duration before dividing by attempts. The existing benchmark instead retains the latest draw duration until each present and can omit an earlier unpresented draw. The same 599 draws occur in both arms, so that reporting distinction does not explain the direction; it changes the reproduced delta from 0.537106 to 0.517408 ms/attempt. The benchmark is unchanged here.

Of that all-draw elapsed increase, only **0.057875 ms/attempt** appears as additional sampled CPU in draw/present. The remaining **0.459532 ms/attempt** is elapsed-minus-sampled-CPU: off-CPU scheduling/waiting and sampling attribution error, not 0.46 ms of identified new CPU instructions. The profile does **not** identify its specific wait or establish a frequency, core-placement or scheduling cause. No such mechanism is claimed. CPU-weighted profile totals agree with measured-window getrusage to within 0.001 ms/attempt in each arm.

### Inclusive function attribution

These are sampled CPU times per attempted present. Full-process input-update rows are separated from draw/present descendants; nested rows overlap.

| Function | Caller/group | Off ms/attempt | On ms/attempt | Change |
|---|---|---:|---:|---:|
| Window::draw | GPUI Fast | 0.675381 | 0.676901 | +0.001520 |
| Window::draw_roots | GPUI Fast | 0.607556 | 0.631081 | +0.023526 |
| Window::present / MetalRenderer::draw | GPUI Fast/Metal | 0.099584 | 0.155940 | +0.056355 |
| TextElement::prepaint | GPUI Kit, draw descendant | 0.338115 | 0.415340 | +0.077225 |
| SyntaxHighlighter::styles | GPUI Kit, draw descendant | 0.075226 | 0.145641 | +0.070415 |
| paint_line | GPUI Fast, draw descendant | 0.065557 | 0.074556 | +0.008999 |
| SyntaxHighlighter::update_edits | GPUI Kit, input handler | 9.213383 | 1.224142 | -7.989241 |
| compute_injection_layers_observed | GPUI Kit, input handler | 8.514341 | 0.422208 | -8.092133 |

`update_edits` and injection discovery have **zero sampled CPU with a draw/present ancestor** in both captures. The source confirms the call path: `input_adapter.rs:160` calls the synchronous update from input handling; `highlighter.rs:711` replaces only injection discovery with negative-result reuse. `styles` remains the existing visible-range highlighting query at `highlighter.rs:1234`; its code is identical with the switch off/on. The extra ~0.070 ms sampled there is observed existing draw work, not a moved injection scan. The broad draw CPU total stays nearly unchanged because other draw descendants decrease; the present encoding subtree grows ~0.056 ms. One capture per arm cannot establish causation for these smaller per-function changes.

### Largest growing self-CPU functions under draw/present

Sampled self times are disjoint; small values are noisy at 1,000 Hz. No unresolved native address is renamed by guesswork.

| Function | Off ms/attempt | On ms/attempt | Change |
|---|---:|---:|---:|
| `gpui::text_system::line::paint_line` | 0.023241 | 0.045627 | +0.022386 |
| `ts_tree_cursor_child_iterator_next` | 0.000466 | 0.015972 | +0.015506 |
| `ts_tree_cursor_current_status` | 0.004370 | 0.019810 | +0.015440 |
| `<gpui::scene::BatchIterator as core::iter::traits::iterator::Iterator>::next` | 0.000000 | 0.011984 | +0.011984 |
| `<gpui_component::highlighter::registry::SyntaxColors>::style` | 0.002471 | 0.014066 | +0.011595 |
| `<ropey::iter::Chunks>::next_impl` | 0.000000 | 0.007363 | +0.007363 |
| `ts_query_cursor__advance` | 0.019213 | 0.026482 | +0.007270 |
| `UNRESOLVED libsystem_malloc.dylib+0x3ff84` | 0.000000 | 0.007249 | +0.007249 |
| `ts_tree_cursor_goto_parent` | 0.000000 | 0.006836 | +0.006836 |
| `UNRESOLVED IOGPU+0x20b48` | 0.000000 | 0.006835 | +0.006835 |

### Actual presentedTime diagnostics

Existing state-version correlator; display time is MTLDrawable.presentedTime, not submit time. Versions with no valid displayed drawable are counted separately. These two profiled captures are descriptive and cannot replace matched five-pair latency comparisons.

| Metric | Off | On |
|---|---:|---:|
| Input → display median ms | 56.299729 | 50.398021 |
| Input → display p95 ms | 64.575750 | 57.838417 |
| Input → display max ms | 68.792833 | 58.232708 |
| Distinct input versions displayed | 522.000000 | 562.000000 |
| Painted input versions without a valid display | 78.000000 | 38.000000 |
| Distinct versions displayed per second | 8.692620 | 9.359956 |
| GPU-complete → presented median | NOT MEASURED | NOT MEASURED |
| Missed refreshes median / p95 against control floor | NOT MEASURED | NOT MEASURED |
| Rendered frames never shown (all draws, including repeated versions) | NOT MEASURED | NOT MEASURED |
| nextDrawable calls >1 ms | NOT MEASURED | NOT MEASURED |
| Display actual refresh rate | NOT MEASURED | NOT MEASURED |

The new G5 control-floor and queue diagnostics were not enabled in these PX-3b captures; sparse presentation cadence is not substituted for actual display refresh. The input-version non-presentation count is not silently relabeled as all rendered frames never shown. These missing fields preclude any G5 acceptance claim.

## Disposition

No avoidable additional **candidate** draw path was identified: the candidate removes input-handler injection scanning and does not schedule work in drawing. Adding a style cache, changing scheduler/QoS or changing presentation would be separate behavior changes without evidence that they remove this residual. No runtime fix is made, so the conditional post-fix five-pair retest is not triggered. PX-3's prior GAIN verdict is unchanged; this diagnostic pair gets no P verdict. The fast path remains proposed and **off by default**; the owner decides its default.

The precise off-CPU source of the elapsed residual remains unverified. The historical ~0.85 ms cannot be causally assigned from these two later CPU profiles; the reproduced ~0.52 ms increase is accounted for above without calling the unexplained residual CPU work or moved highlighting.

## Validation

- Pinned release build with symbols: exit 0.
- Both 60-second profile captures: exit 0, validated 600 edits each, unchanged binary hash.
- `scripts/check.sh`: exit 0; seven groups total 1,875 tests, 1,868 passed and 7 ignored (RustUI 1,456 + 7 ignored; core 11; i18n 86; macros 3; Lab 40; Playground 272; RGPE 0).
- `./scripts/verify-release.sh source`: exit 0, 695 tests passed.
- Focused G0 test-window selection: 2 passed.
- Active damage stack `--check`: exact postimages pass.
- Source CI and PR review are required before merging into `p-integration`.
