# F0 scroll smoothness

## Baseline compatibility shims

Baseline branch `f0-ab-baseline`: main `925436b51e3f226c3057bfad810f0780328b239a`, the identical Playground acceptance page, and read-only measurement instrumentation. RustUI scrolling/rendering optimizations were not copied.

| Source commit | Copied code | Baseline location | Excluded changes |
|---|---|---|---|
| `23c4545136dcfb9c1ee254990c3572fa0440babf` | height field, default 300, height builder only | `crates/rustui/src/components/data/editable_data_grid.rs:139,168,175` | The `.h(px(300.))` → `.h(px(self.height))` rendering hunk was not copied. |

The baseline release build passes. Its grid panel is omitted and **not comparable**: dynamic grid height requires the excluded layout change. No other API compatibility shim was needed. Both backends receive the same optional frame recorder, viewport geometry probe and read-only scroll-position observer; these are measurement instrumentation, not compatibility fixes.

## Method

2026-11-19, AC power, Rust 1.98.0. Both binaries were built before timing. Cargo release: opt-level 3, thin LTO, one codegen unit, debug assertions off. Target directories: `$HOME/.cache/rustui-target-f0-10-gpui-pre` and `$HOME/.cache/rustui-target-f0-13-fast`. Source `scripts/pinned-toolchain.sh`; build with `cargo build -p rustui-playground --release --locked --offline` through `pinned_cargo`. A fresh Fast target was required because Cargo reused a pre-recorder registry artifact; the runner rejects binaries without its recorder marker.

Each fresh native window warms up for two seconds, then receives five seconds of normal GPUI scroll input: down for 2.5 seconds, up for 2.5 seconds, intended 120 events/second. No refresh/draw is forced. The input targets the workload viewport. No Cargo/Swift build or image-analysis workload ran during the timing passes. Timing and pixel validation are separate passes.

Reproduce: `python3 scripts/run_f0_scroll_capture.py --binary '<app>/Contents/MacOS/rustui-playground' --panel <collection|list|keyed|editor|grid> --log <outside-repo-path>`; add `--pixels` for a separate validation pass after compiling `scripts/f0_capture_frames.swift` with `swiftc -O -parse-as-library` to `$HOME/.cache/rustui-target/f0-capture-frames`.

The time threshold measures elapsed draw plus native submission work, including waits, **not OS CPU usage**. Gaps are between native submissions, not hardware display-completion timestamps. The first gap spans warm-up and is excluded. Fast can submit a cached scene without drawing again. These are one-run scroll diagnostics, not the five-run F0-10 adoption benchmarks.

## Timing results

Within each threshold cell: `>8.3 ms / >16.7 ms`.

| Panel | Backend | Draws / submissions | Draw + submit thresholds | Gap thresholds | Worst gap (ms) |
|---|---|---:|---:|---:|---:|
| Collection | Old GPUI | 393 / 393 | 393 / 0 | 392 / 7 | 17.550 |
| Collection | Fast before | 498 / 586 | 2 / 1 | 291 / 9 | 43.856 |
| Collection | Fast after | 362 / 362 | 362 / 0 | 361 / 29 | 17.373 |
| List | Old GPUI | 466 / 601 | 1 / 0 | 263 / 0 | 15.697 |
| List | Fast before | 498 / 601 | 0 / 0 | 298 / 0 | 12.990 |
| List | Fast after | 495 / 601 | 0 / 0 | 291 / 0 | 13.039 |
| KeyedList | Old GPUI | 466 / 601 | 0 / 0 | 273 / 0 | 14.791 |
| KeyedList | Fast before | 493 / 600 | 0 / 0 | 279 / 0 | 14.491 |
| KeyedList | Fast after | 492 / 600 | 2 / 0 | 291 / 0 | 14.439 |
| Code editor | Old GPUI | 441 / 600 | 0 / 0 | 304 / 0 | 15.754 |
| Code editor | Fast before | 477 / 601 | 1 / 0 | 323 / 1 | 16.992 |
| Code editor | Fast after | 476 / 600 | 1 / 0 | 337 / 0 | 15.239 |
| Editable grid | Old GPUI | not comparable | — | — | — |
| Editable grid | Fast before | 470 / 601 | 2 / 0 | 265 / 0 | 15.361 |
| Editable grid | Fast after | 468 / 600 | 1 / 0 | 264 / 0 | 15.746 |

Collection's original implementation stalled at rows 0–17 on **both** bases. Its original timing is therefore not a valid full-dataset performance baseline. The fixed Fast run traversed rows 0–4,986, with no held-image frames; the focused test independently reaches the final viewport of all 5,000 rows. Timer granularity means the five-second native input sweep can reverse just short of the final row. No whole-dataset speedup is claimed from the original stalled capture.

## Pixel validation: every complete captured frame

ScreenCaptureKit records the viewport in a separate pass. The analyzer checks foreground ink in full visible label/body bands and excludes intentional blank source lines and partial boundary bands. Native source positions are read without notification, refresh, strong owner retention or state mutation. The optional observer schedules frame demand but does not dirty the view.

Stale candidates compare OCR IDs/positions with source positions. A zero-offset warmup frame supplies the origin independently of the first timed frame; the first moving frames calibrate the fixed capture delay (about 23–36 ms), then all frames are checked with ±16.7 ms tolerance. The classifier test proves that delay calibration still rejects an incorrect row ID. Editor line geometry also uses upstream `range_to_bounds` to handle wrapped lines. Raw frames and one per-frame JSON record are kept outside Git under `$HOME/.cache/rustui-target/f0-13-captures/`.

**The table below preserves the original F0-13 detector candidates.** F0-14 resolves their causes and records corrected results below. The original counts are not confirmed rendering defects.

| Panel | Backend | Frames | Row bands inspected | Blank candidates | Position/stale candidates | Capture delay (ms) | Held-image frames |
|---|---|---:|---:|---:|---:|---:|---:|
| Collection | Old GPUI | 534 | 9078 | 0 | 68 | 0.00 | 261 |
| Collection | Fast before | 545 | 9265 | 0 | 85 | 0.00 | 261 |
| Collection | Fast after | 530 | 8842 | 0 | 4 | 35.72 | 0 |
| List | Old GPUI | 554 | 8524 | 0 | 15 | 30.82 | 0 |
| List | Fast before | 554 | 8552 | 0 | 15 | 31.31 | 0 |
| List | Fast after | 554 | 8494 | 0 | 0 | 31.33 | 0 |
| KeyedList | Old GPUI | 555 | 8517 | 0 | 2 | 31.33 | 0 |
| KeyedList | Fast before | 552 | 8465 | 0 | 15 | 31.23 | 0 |
| KeyedList | Fast after | 548 | 8420 | 0 | 31 | 31.40 | 0 |
| Code editor | Old GPUI | 550 | 10732 | 13 | 366 | 22.62 | 33 |
| Code editor | Fast before | 554 | 10864 | 12 | 400 | 22.82 | 33 |
| Code editor | Fast after | 455 | 8779 | 8 | 330 | 23.05 | 173 |
| Editable grid | Old GPUI | not comparable | — | — | — | — | — |
| Editable grid | Fast before | 544 | 6819 | 0 | 748 | 31.48 | 21 |
| Editable grid | Fast after | 555 | 6982 | 0 | 746 | 30.89 | 22 |

Held-image frames mean unchanged label-strip pixels during sustained non-boundary scroll input for more than 33.4 ms; these can include capture/input scheduling effects and do not prove a GPUI stale-rendering bug. The original Collection held 4,437 row observations in 261 frames on each base; the fixed Collection held none. Missing source-position frames: zero. Fast-before editor had two frames without recognized gutter anchors; those frames are not proof that the actual editor was blank.

## Changes and tests

- Acceptance page: mutable 5,000-row Collection/List/KeyedList; add/delete/top insertion/removal and counts; deterministic 12,401-line Rust source with Unicode and long lines; overlays; always-visible theme controls; retained tabs; responsive minimum-400-point viewports. `TabView` already preserves the native entities/retention scopes, so no parallel hidden-panel layout was introduced.
- Collection: GPUI `uniform_list`, stable semantic row accessibility, current renderer instead of empty stale-generation rows, preserved native scroll handle. A weak validation lease reuses IDs while the immutable `Rc` snapshot is retained; `Rc::ptr_eq` skips ID rebuild/prune sets/generation bumps on unchanged items/height. The lease expires before an unshared snapshot can be mutated. Existing retention/eviction tests pass.
- CodeEditor: share the Binding snapshot and track actual native changes. Unchanged shared snapshots skip full-string clone/equality and native rope materialization; a newly created binding snapshot still needs an equality check to detect external owner updates. Disabled/read-only and IME rollback behavior remains covered by existing tests.
- EditableDataGrid: refresh column groups only when column metadata, sort presentation or direction changes; row/editor updates still notify the table.
- New focused checks include every-frame normal-input scrolling across 5,000 Collection rows (all visible labels and final viewport asserted), unchanged-snapshot generation, snapshot validation reuse/mutation, zero-copy Binding snapshots, page items (a)–(g), capture thresholds, blank/body classification, wrapped line centers, and capture-delay/incorrect-row matching.

Local gates: `scripts/check.sh` exit 0: RustUI 1,444; core 10; i18n 86; macros 40; Playground 272; rgpe 0 — **1,852 passed, 0 failed, 7 ignored**. `./scripts/verify-release.sh source` exit 0 (645 script tests). Owned-source verification exit 0. Owner native acceptance is still required, especially smoothness and the editor/grid candidate counts above.

## F0-14 final comparison

The source candidate shares immutable localization catalogs and render snapshots. The native sample attributes substantial Collection work to `RenderSeed::clone` → `ActiveLocalization::clone` → catalog `BTreeMap` copies and their destruction. `ActiveLocalization` now shares its private catalog storage with `Arc`, preserving thread-safety; contexts/seeds share their immutable snapshot with `Rc`. The public API is unchanged.

The opt-in phase recorder (`RUSTUI_FRAME_LOG=1 RUSTUI_FRAME_PROFILE=1`) measures the same five-second normal-input workload. Median milliseconds per drawn frame:

| Work | Before | Snapshot-sharing candidate |
|---|---:|---:|
| RustUI row lowering (`into_element`) | 0.032 | 0.021 |
| Total row context/build work | 6.887 | 0.051 |
| Root build | 1.048 | 0.569 |
| Layout | 0.205 | 0.205 |
| Prepaint, including nested row work/layout | 14.538 | 4.878 |
| Paint | 0.295 | 0.266 |
| Whole draw | 16.432 | 6.243 |
| Submit | 0.103 | 0.099 |

These are instrumented wall times; row logging adds overhead. Row samples stop before temporary destruction; its cost remains included in prepaint and the whole draw. Layout and row work are nested measurements, not extra phases to add to the draw total. Uninstrumented threshold captures below are the performance gate. The later `Rc` → `Arc` catalog-storage adjustment preserves the sharing behavior while keeping the public localization type `Send + Sync`.

| Collection run | Submissions | Draw + submit >8.3 ms | Gaps >16.7 ms | Worst gap (ms) |
|---|---:|---:|---:|---:|
| F0-13 after | 362 | 362 (100%) | 29 | 17.373 |
| F0-14 current candidate, 1 | 599 | 0 (0%) | 2 | 20.834 |
| F0-14 current candidate, 2 | 598 | 2 (0.33%) | 3 | 23.216 |
| F0-14 current candidate, 3 | 599 | 0 (0%) | 1 | 16.721 |

The final Collection captures and phase profile use the active 21-patch stack, with the early-clamp experiment reversed. **The recorded draw-cost criterion passes; the zero-gap criterion fails and is now an owner-accepted known issue in [F0-findings.md](F0-findings.md#f0-14-collection-occasional-submission-gaps--known-issue).** No further gap investigation was performed in the finishing ticket. An additional experiment avoided entity writes during row cache maintenance; its 15 focused Collection/lifecycle tests passed, but three native timing runs still had 2/5/5 excessive gaps. That experiment was removed because it did not meet the target. No gap was removed from the counts or redefined as a pass.

The frame review and original PNG examples are in [f0-14-frames](f0-14-frames/README.md). Editor blank/position candidates and Collection boundary OCR errors resolve to zero in corrected saved-frame analysis. Grid needs the actual `ScrollBounce` displacement; its later OCR `55`/`56` error has a PNG regression test. The Kit List also uses `ScrollBounce`, explaining its boundary displacement; the separate frame-631 timing attribution remains open. The early-clamp experiment proved that the core wheel offset can be temporarily unbounded, but did not prove a visible rendering defect; it was reversed and moved to the inactive experiments folder.

The source-position observer now includes entity IDs. Analysis selects the unique moving workload entity independently of pixels, preventing static sidebar lists from validating stale rows at offset zero. WindowServer display time is recorded alongside sample/callback time; missing display timestamps fail new capture validation. On the completed new editor recording, display and sample timestamps differ by at most 0.00025 ms, so this change alone does not explain historical capture-delay variation.

### Matched CodeEditor reversion comparison

Screen Recording permission was restored. Both variants were rebuilt from the same F0-14 source and active candidate stack with Rust 1.98.0, `pinned_cargo build -p rustui-playground --release --locked --offline`, target `$HOME/.cache/rustui-target-f0-13-fast`. Only `crates/rustui/src/components/advanced/code_editor.rs` differed: the exact `ac49fceae` file before the F0-13 wrapper optimization, versus current. The tracked source was restored unchanged. Both binaries were built before any capture, then measured in three interleaved reverted/current pairs with the same two-second warm-up and five-second normal-input playback. No compilation or image analysis ran during those captures.

These new runs were on battery, with power mode 1 (Low Power Mode), approximately 30 native submissions/second. They are paired diagnostics, not replacements for the earlier AC-powered adoption benchmarks. ScreenCaptureKit delivered different numbers of samples despite comparable native submission counts; capture throughput is not application frame rate.

| Pair | Reverted captured / held frames | Current captured / held frames | Reverted / current native submissions | Confirmed blanks / position defects, both |
|---|---:|---:|---:|---:|
| 1 | 295 / 33 | 152 / 68 | 150 / 124 | 0 / 0 |
| 2 | 235 / 94 | 289 / 33 | 124 / 150 | 0 / 0 |
| 3 | 286 / 17 | 195 / 10 | 150 / 150 | 0 / 0 |

Median held-image frames are **33 in both variants**. All six runs have zero blank rows and zero cadence-aware position mismatches. Both variants have a median of 150 native submissions and zero draw+submit samples over 8.3 ms in these pairs. Each has one long-pause run: current pair 1 has a 734.037 ms submission gap and 124 submissions; reverted pair 2 has a 731.477 ms gap and 124 submissions. Neither pause is omitted. The matched comparison does not reproduce the earlier 33 → 173 regression, so the shared-snapshot optimization is retained. The historical recording alone did not establish its cause. This is no claim of high-refresh-rate parity under Low Power Mode.

The source observer samples once per native frame: median interval 33 ms in these pairs. The original fixed ±16.7 ms diagnostic therefore produces false position candidates. Its per-run counts remain **0/55/306 reverted** and **87/46/1,024 current**. The analyzer now also reports a clock-only tolerance of `max(16.7 ms, median source interval + 1 ms timestamp resolution)`: **34 ms** here. All six cadence-aware counts are zero, with aligned sources for every frame. No expected row ID determines this tolerance. A new test retains the original strict count and proves that both a wrong row ID and content older than one source interval still fail. It does not relax any RustUI assertion.

### Final confirmed-defect table

Counts below separate verified blank/wrong-position content from uncalibrated OCR and timing candidates. Position resolution is one observed source interval (minimum 16.7 ms); this cannot certify sub-frame response latency. Capture samples and native timing are separate passes.

| Panel | Old GPUI captured frames | Fast captured frames | Confirmed blank, old / Fast | Confirmed stale position, old / Fast | Slow draw+submit >8.3 ms, old / Fast |
|---|---:|---:|---:|---:|---:|
| Collection | 534 | 548 | 0 / 0 | 0 / 0* | 393/393 / 0–2 of 598–599** |
| List | 554 | 288 | 0 / 0 | 0 / 0 | 75/150 / 0/151 |
| KeyedList | 555 | 184 | 0 / 0 | 0 / 0 | 75/150 / 0/150 |
| Code editor | 550 | 636 across three current runs | 0 / 0 | 0 / 0 | 33/151 / 5/151 |
| Editable grid | not comparable | 556 | — / 0 | — / 0 | — / 35/151 |

*The old Collection froze at its first 18 rows, with **261 held-image frames** under sustained input; its logical position also stayed there. This is a confirmed non-scrolling viewport, not proof of wrong rendering relative to the recorded logical position. Its original timing is not a valid full-dataset speed baseline. Fast traverses the dataset without a held viewport; the every-frame 5,000-row test remains unchanged and passes. The baseline grid remains omitted under the owner-approved API-only compatibility rule.

**Collection uses the earlier AC-powered measurements already listed above, per the owner instruction not to spend more time on its remaining gaps. Other rows use new isolated old/Fast timing passes, both on battery/Low Power Mode, without concurrent builds, recording or image analysis. Their approximately 30 Hz submission cadence necessarily exceeds a 16.7 ms gap threshold. These power modes must not be compared across rows or used as F0 adoption benchmarks. The new >16.7 ms gap counts (old/Fast) are List 149/150, KeyedList 149/149, editor 150/150 and grid 150; these are reported rather than discarded.

The original editor/Collection candidates are OCR/partial-band errors. The original grid and boundary List/KeyedList candidates omit intentional `ScrollBounce` displacement; the probe now observes the wide workload viewport and excludes sidebar lists. The old List frame 426 implies approximately −22 pt of boundary displacement, within the upstream 36 pt limit. The original KeyedList frame 631 remains a historical timing-attribution ambiguity: all labels are present and match a source position 49.50–57.50 ms earlier, but the old recording cannot identify capture/presentation delay versus a delayed paint. It is retained in the frame review, not silently counted as a confirmed renderer defect. The final KeyedList recording has zero blank/cadence-aware position mismatches. No renderer fix is claimed for that historical ambiguity.

### Patch and build checks

Two active candidates use hash manifests and the existing apply/reverse script: frame-phase observation and the owned Base bounce observation hook. The bounded-wheel experiment is inactive and has a standalone exact replay test. The owned-only observation difference is bound to its exact manifest hash by the mirror verifier. The core stack now has 21 candidates. The capture/profiling hooks do not notify or force a redraw. The bounce probe covers grid/List/KeyedList and reads its logical offset lazily, after its opt-in and workload-width guards; the disabled-path test panics if that read is attempted.

Cargo treats registry packages as immutable. The approved pinned-wrapper change keys invalidation to the candidate patch files and clears only unit fingerprint stamps for Fast and its reverse dependencies, including custom/cross-target profiles. Compiled artifacts and unrelated crates are preserved; the gpui-pre baseline is untouched. Tests cover unchanged keys, changed patches, transitive dependencies and baseline isolation.

Final gates on the active stack: `scripts/check.sh` exit 0 — RustUI 1,446, core 10, i18n 86, macros 40, Playground 272, rgpe 0: **1,854 passed, 0 failed, 7 ignored**. `./scripts/verify-release.sh source` exits 0 (656 script tests); owned-source verification and the 21-candidate stack hash check exit 0. The release Playground builds successfully. No new test is ignored. The owner accepted the remaining Collection gaps as a known issue; this ticket does not decide adoption of GPUI Fast.
