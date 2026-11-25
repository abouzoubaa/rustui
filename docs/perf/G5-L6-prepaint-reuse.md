# G5-L6 visible editor prepaint reuse

## Change and oracle

`RUSTUI_G5_PREPAINT_REUSE=1` retains exact previous-frame visible text
layouts in GPUI Kit. The cache checks source bytes, styled runs clipped
relative to each line, wrap boundaries and indentation, plus window, scale,
width, font, font size, tab and whitespace settings. Theme changes that
alter a line’s styled runs invalidate that line. After scrolling, newly
visible rows are shaped and unchanged rows already in the cache can be
reused. Multiline edits can reuse unchanged visible rows at shifted line
indices. Placeholder, masked, token-inline and document-color swatch
layouts use the existing path.

A row hit allocates no text, run vector, or shaped layout. A fully
unchanged viewport shares the prior array. A frame with changed rows may
allocate one new array of references and the cache entries for misses.
Syntax-style computation, caret and gutter work still run normally.

`RUSTUI_G5_PREPAINT_VERIFY=1` compares eligible cached line layouts with
full reshaping, including glyph geometry, styled runs and wrap layout.
The focused test covers insert, delete, Paste, Undo, scroll, width and
theme changes.

## Five paired native captures

The measurement used the 10,000-line RustUI editor, 10 posted key
events/s delivered through the native Cocoa path, direct launch, display
sync off, damage off, buffered logging and passive link off. Each pair used 60 seconds after warmup on
one release build, alternating off-first and on-first. The L4 phase and
L2 display-sync patches were applied to that build for measurement and
reversed from the branch. All ten logs reported sync disabled throughout
the measured window. Handler and GPU-density probes were disabled for
the latency pairs. The separate diagnostic captures used those probes.

The L4 miss-rate proxy counts a key when the first matching
ScreenCaptureKit frame arrives at least 8.333 ms after GPU end. Prepaint excludes nested layout
compute. P is the native handler to Metal GPU end. Median/p95 values in
the table are milliseconds. Process CPU uses the capture’s getrusage
user and system delta. The P verdict uses CPU per attempted present.

| Metric | Off | Reuse | Target |
| --- | ---: | ---: | --- |
| Prepaint | 0.724 / 0.956 | 0.550 / 0.760 | median ≤ 0.250 ms |
| P | 4.442 / 6.405 | 4.280 / 6.124 | median ≤ 3.800 ms |
| Handler → SCK | 8.868 / 19.155 | 8.548 / 18.405 | — |
| Handler → Metal presentedTime (median / p95 / max) | 8.846 / 30.868 / 46.098 | 8.331 / 30.399 / 46.909 | — |
| GPU end → Metal presentedTime (median) | 4.287 | 3.755 | — |
| Median distinct versions shown/s | 9.76 | 9.84 | — |
| Metal submissions without positive presentedTime | 0 | 0 | — |
| Miss rate | 33.64% | 31.37% | — |
| Median process CPU per attempt | 7.034 ms | 6.853 ms | INCONCLUSIVE |
| Median process CPU per second | 0.06873 s/s | 0.06743 s/s | INCONCLUSIVE (diagnostic) |

Median prepaint fell 0.174 ms and median P fell 0.162 ms. Both missed
the absolute targets. Process CPU did not meet the five-pair rule for
a gain or a regression.

| Pair | Corrected background precheck | Capture start/end CPU, off and on | Prepaint median off / on | CPU per attempt change |
| --- | ---: | --- | ---: | ---: |
| 1 | 2.64% | 3.91% / 22.57% · 3.56% / 22.46% (loaded, peak snapshot 22.57%) | 0.716 / 0.551 ms | -4.61% |
| 2 | 3.67% | 22.55% / 16.60% · 2.91% / 17.66% (loaded, peak snapshot 22.55%) | 0.730 / 0.561 ms | -4.75% |
| 3 | 13.48% | 13.35% / 23.13% · 12.53% / 22.71% (loaded, peak snapshot 23.13%) | 0.723 / 0.540 ms | -0.37% |
| 4 | 13.01% | 12.68% / 26.06% · 12.99% / 22.77% (loaded, peak snapshot 26.06%) | 0.720 / 0.545 ms | -1.87% |
| 5 | 20.56% | 21.31% / 22.31% · 14.43% / 22.63% (loaded, peak snapshot 22.63%) | 0.732 / 0.551 ms | -1.61% |

CPU per attempt pair changes were -4.61%, -4.75%, -0.37%, -1.87%, -1.61%. Median -1.87%, range -4.75% to -0.37%.

CPU per second pair changes were -3.63%, -3.82%, +0.27%, -2.38%, -1.39%. Median -2.38%, range -3.82% to +0.27%.

The reuse arm reused 107,820 of 110,815 visible-row layout requests (97.30%).
The off arm had 63 unjoined keys, 0 lost edits and 0 order violations.
The on arm had 48 unjoined keys, 0 lost edits and 0 order violations.
Per-capture load exceeded 5% at 7/10 starts and 10/10 ends. The pair label accounts for the pre-pair check and both captures’ start and end snapshots.

The passive display link was off. The SCK observer supplies timestamps for decoded content changes, so physical refresh cadence and missed-refresh counts against a control floor were not measured in these captures.

`nextDrawable` exceeded 1 ms once in the measured windows, with zero calls over 2 ms.
Two warmup calls also exceeded 1 ms.

| Pair | Arm | Capture phase | Call | Path | In flight | Since link detach |
| --- | --- | --- | ---: | --- | ---: | ---: |
| 1 | off | warmup | 1.001 ms | non-immediate | 0 | — |
| 3 | off | measured | 1.422 ms | immediate | 1 | 22217.882 ms |
| 4 | on | warmup | 1.075 ms | non-immediate | 0 | — |

## Handler subphases

The RustUI diagnosis uses one separate 60-second native capture, with
reuse off. The spans below are joined to the native key and stop at the
outer handler boundary. The inclusive `replace_text_in_range` scope is
excluded from this ranking because it overlaps its children.

| Measured scope | File or function | Median | p95 | Keys with span |
| --- | --- | ---: | ---: | ---: |
| Wrap and display-map projection | `DisplayMap::adjust_folds_for_edit / on_text_changed` | 0.142 ms | 0.232 ms | 585 |
| Text, annotations and history | `InputBaseState::replace_text_in_ranges / push_history` | 0.012 ms | 0.021 ms | 585 |
| Highlighter update | `LayoutMode::update_highlighter` | 0.011 ms | 0.020 ms | 585 |

The markers are in `crates/rustui-gpui-base/src/input/base/state.rs`.
The display-map methods are in
`crates/rustui-gpui-base/src/input/editor/display_map/display_map.rs`.
These scoped times leave AppKit dispatch and any unmarked Kit work
unattributed. The edit path was instrumented without moving work.

## GPU density

One 60-second native capture per app with the same renderer-density
probe. Median per joined key. Draw calls count encoded Metal draw API
calls in the frame, including path intermediates and layer draws if
present. Coverage sums viewport- and content-mask-clipped primitive
bounds in device pixels, counting overlap per primitive. The union
snaps covered bounds outward to 16 px tiles. Rounded masks, glyph
silhouettes, early discards, and shader efficiency can change actual
fragment work. These are geometry estimates.

| Metric | RustUI | Bare GPUI Fast |
| --- | ---: | ---: |
| quads | 36 | 27 |
| shadows | 0 | 0 |
| paths | 1 | 0 |
| underlines | 0 | 0 |
| mono sprites | 878 | 2 |
| subpixel sprites | 0 | 0 |
| poly sprites | 0 | 0 |
| surfaces | 0 | 0 |
| draw calls | 11 | 2 |
| drawable px | 3,840,000 | 3,840,000 |
| coverage sum px | 7,574,405 | 3,853,694 |
| coverage tile union px | 3,840,000 | 3,840,000 |
| quad px | 7,371,664 | 3,853,312 |
| shadow px | 0 | 0 |
| path px | 43,301 | 0 |
| underline px | 0 | 0 |
| mono px | 159,440 | 382 |
| subpixel px | 0 | 0 |
| poly px | 0 | 0 |
| surface px | 0 | 0 |
| gpu ms | 0.679 | 0.324 |

GPU density joined 585 RustUI keys and 600 bare keys. Both diagnostic
captures were loaded. Corrected background CPU at start/end was
12.62% / 23.03% for RustUI and 12.68% / 23.37% for bare GPUI.

The 0.679 ms versus 0.324 ms GPU medians reproduce the roughly twofold
difference. Both apps covered the same 3.84 million-pixel drawable.
RustUI’s quads alone covered 7.37 million pixels when overlap is counted,
compared with 3.85 million in bare GPUI. The editor also issued 878 glyph
sprites and 11 draw calls, compared with 2 glyph sprites and 2 draw calls
in the single text input. The added work is consistent with the difference
in GPU execution time. The probe cannot attribute specific milliseconds
to overdraw, glyphs or individual draw calls.

Per-key diagnostic counts and intervals are saved in
`target/headstart/g5-l6/diagnosis-per-key.csv`. The renderer probe uses
`MetalRenderer::draw` and the draw-call sites in `metal_renderer.rs` and
the corresponding path and layer helpers. No renderer optimization is
included.
