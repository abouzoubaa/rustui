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

Stale candidates compare OCR IDs/positions with source positions. The first moving frames calibrate the fixed capture delay (about 23–36 ms), then all frames are checked with ±16.7 ms tolerance. The classifier test proves that delay calibration still rejects an incorrect row ID. Editor line geometry also uses upstream `range_to_bounds` to handle wrapped lines. Raw frames and one per-frame JSON record are kept outside Git under `$HOME/.cache/rustui-target/f0-13-captures/`.

**Counts below are detector candidates, not confirmed rendering defects.** OCR can misread line numbers; grid bottom/clipping geometry also produces position mismatches. Inspected editor candidates showed painted content. Editor/grid confirmed blank/stale counts remain **NOT VERIFIED**; zero candidates on Collection/List/KeyedList means no detected blank label in the captured frames, not a universal flicker-free guarantee. Pixel passes include capture/observer overhead and are not used as timing results.

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

Local gates: `scripts/check.sh` exit 0: RustUI 1,444; core 10; i18n 86; macros 40; Playground 272; rgpe 0 — **1,852 passed, 0 failed, 7 ignored**. `./scripts/verify-release.sh source` exit 0 (644 script tests). Owned-source verification exit 0. Owner native acceptance is still required, especially smoothness and the editor/grid candidate counts above.
