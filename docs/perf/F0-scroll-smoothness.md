# F0 scroll smoothness

Work in progress. The acceptance page and Fast timing recorder are implemented; the A/B comparison and scrolling optimizations are not complete.

## Comparison blocker

The baseline is 925436b5 plus only the Playground portions of the acceptance-page commits. Its release build exits 101 at apps/rustui-playground/src/stories/editable_grid_gallery.rs:101: `no method named height found for struct EditableDataGridView<R>` (E0599). The viewport-height adapter was introduced after that baseline in RustUI, outside the Playground. The owner was asked whether to add that small adapter to the comparison branch; no adapter has been applied. This is required to reconcile the identical-page requirement with the Playground-only cherry-pick restriction.

No Collection, CodeEditor, or EditableDataGrid scrolling optimization has been applied: the owner requires the before measurements first.

## Preliminary Fast capture

2026-11-19, AC power, AC powermode 0. Rust 1.98.0, Cargo release profile (opt-level 3, thin LTO, one codegen unit, debug assertions off). A fresh backend target directory was required: Cargo reused a pre-instrumentation registry artifact in the shared target directory. The capture tool rejects binaries lacking the recorder marker.

Build: `CARGO_TARGET_DIR=$HOME/.cache/rustui-target-f0-13-fast rustup run 1.98.0 cargo build -p rustui-playground --release --locked --offline`, after sourcing scripts/pinned-toolchain.sh.

For each panel the native app warms up for two seconds, then dispatches normal scroll input for five seconds, downward for 2.5 seconds and upward for 2.5 seconds, at an intended 120 events/second. It never forces a redraw. No Cargo build ran during these captures. Reproduce with `python3 scripts/run_f0_scroll_capture.py --binary '<bundle>/Contents/MacOS/rustui-playground' --panel <collection|list|keyed|editor|grid> --log <outside-repository log>`.

The time column measures elapsed drawing plus native submission work, including waits; it is not an operating-system CPU-usage counter and excludes input handling outside drawing. Gaps are between native frame submissions, not hardware display-completion timestamps. The first gap is excluded because it spans the warm-up idle period. Presentations can outnumber draws when Fast reuses its scene. These are preliminary Fast-only observations, not an official old-versus-Fast result.

| Panel | Old GPUI | Fast draws / submissions | Draw + submit >8.3 ms / >16.7 ms | Submission gaps >8.3 ms / >16.7 ms | Worst gap (ms) | Fast after | Empty visible rows |
|---|---|---:|---:|---:|---:|---|---|
| Collection | NOT VERIFIED | 514 / 596 | 2 / 1 | 301 / 2 | 27.284 | NOT VERIFIED | NOT VERIFIED |
| List | NOT VERIFIED | 511 / 601 | 0 / 0 | 299 / 0 | 13.358 | NOT VERIFIED | NOT VERIFIED |
| KeyedList | NOT VERIFIED | 493 / 598 | 0 / 0 | 282 / 1 | 21.109 | NOT VERIFIED | NOT VERIFIED |
| Code editor | NOT VERIFIED | 478 / 600 | 1 / 0 | 317 / 0 | 14.974 | NOT VERIFIED | NOT VERIFIED |
| Editable grid | NOT VERIFIED | 469 / 601 | 1 / 0 | 256 / 0 | 16.119 | NOT VERIFIED | NOT VERIFIED |

The recorder does not yet count empty painted rows. It deliberately returns null instead of claiming zero. Native scroll-range verification and the every-frame visible-label regression test remain required before claiming that the full dataset was traversed without blanks. The before/after optimization comparison and side-by-side owner launch remain pending.
