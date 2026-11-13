# A3 redraw profile

Profiled the release-mode GPUI test-window workload on Apple M1 Max, macOS 27.0, Rust 1.98.0. Kit 0.8.1 / gpui-pre 0.3.10 is `origin/main` at `746a77fb`; Kit 0.9.1 / gpui-pre 0.3.12 is A3 activation commit `c72861b8`. Each row is the median of three direct runs of the existing ignored `owner_update_and_draw_workload` test binary. The workload is not a native-frame or GPU benchmark.

## Before and after

| Revision | 1,000-row update only | 1,000-row update + forced draw | Three forced-draw run medians | Spread |
|---|---:|---:|---|---:|
| Kit 0.8.1 / gpui-pre 0.3.10 | 4.4715 ms | 8.5179 ms | 8.4700 / 8.7269 / 8.4167 ms | 3.66% |
| Kit 0.9.1 / gpui-pre 0.3.12 | 4.5623 ms | 8.5821 ms | 8.4893 / 8.4439 / 8.6976 ms | 2.99% |

The matched main-to-upgrade differences are +0.0908 ms for update-only and +0.0642 ms for forced draw. Against the recorded A2 result (4.5905 ms update-only, 8.24 ms forced draw), Kit 0.9.1 updates 0.0282 ms faster and forced draw is 0.3421 ms slower. The result is below the owner's accepted 9.0 ms 0.4.0 ceiling. No behavior-changing or GPUI-source optimization was made.

## Time Profiler sample split

Recorded with `xcrun xctrace record --template 'Time Profiler' --launch -- <release test binary> --exact runtime::component_host::component_lifecycle_tests::owner_update_and_draw_workload --ignored --nocapture`. The table classifies the leaf frame of each 1 kHz sample in the final one-second window, which covers the tail of the 1,000-row forced-draw workload. Estimated milliseconds are each sample share multiplied by the independently measured forced-draw median; this is a sampling estimate, not direct phase instrumentation. Owner-update-only time is measured separately above; update and draw are not independently separable in the test's combined forced-draw loop. Allocations, copies and samples without a resolved stack are included in “Other”.

| Sampled phase | Kit 0.8.1 samples / estimate | Kit 0.9.1 samples / estimate |
|---|---:|---:|
| Owner-update function self time | 0 ms resolved; update-only median 4.4715 ms | 2 ms resolved; update-only median 4.5623 ms |
| GPUI layout (Taffy) | 246 / 2.136 ms | 233 / 2.037 ms |
| Prepaint + paint | 116 / 1.007 ms | 93 / 0.813 ms |
| Text layout, shaping and segmentation | 101 / 0.877 ms | 112 / 0.978 ms |
| Other and unresolved stacks | 518 / 4.498 ms | 542 / 4.737 ms |

The profile identifies a new text-wrapping path in the 0.9.1 source: it builds a reusable text buffer, uses `UnicodeSegmentation::grapheme_indices`, and obtains line-break boundaries from `icu_segmenter::LineSegmenter`. Kit 0.8.1 iterated the existing fragments and used its character-width cache and word-character rules. The 0.9.1 tail sample includes 9 ms in `icu_segmenter::LineBreakIterator::next` (none was sampled in the Kit 0.8.1 path); text-layout/shaping samples rise by 11 ms. Layout and paint samples fall by 13 and 23 ms, while memory-copy and miscellaneous/unresolved samples rise. The small matched forced-draw difference is therefore not attributable to ICU alone; text-buffer copying plus ICU/grapheme boundary work is the identifiable new candidate hotspot.

## Top sampled self-time functions

Each value is the approximate number of 1 ms samples attributed to that leaf function in the final one-second window. Names are shortened to the owning function; allocator entries are system functions.

| Rank | Kit 0.8.1 | Samples | Kit 0.9.1 | Samples |
|---:|---|---:|---|---:|
| 1 | `platform_memmove` | 64 | `platform_memmove` | 80 |
| 2 | `taffy::compute::block::compute_inner` | 69 | `taffy::compute::block::compute_inner` | 55 |
| 3 | allocator `_xzm_free_main` | 41 | allocator `_xzm_free_main` | 40 |
| 4 | `gpui::TextSystem::line_wrapper` | 25 | allocator `_xzm_xzone_malloc_tiny` | 27 |
| 5 | Taffy `generate_item_list` iterator | 24 | `TaffyView::compute_child_layout` | 26 |
| 6 | allocator `_xzm_xzone_malloc_tiny` | 23 | `gpui::TextSystem::line_wrapper` | 23 |
| 7 | `gpui::text_system::line::paint_line` | 20 | Taffy `generate_item_list` iterator | 23 |
| 8 | `taffy::compute::flexbox::compute_preliminary` | 18 | `gpui::text_system::line::paint_line` | 22 |
| 9 | `TaffyView::compute_child_layout` | 17 | `gpui::WindowTextSystem::shape_text` closure | 20 |
| 10 | `taffy::compute::block::compute_block_layout` | 16 | `taffy::compute::flexbox::compute_preliminary` | 20 |

Regression to win back in Stage F (GPUI Fast).

## Final A3-7 gate run

The final `scripts/perf-baseline.sh` run completed three release-mode runs on the A3 integration tip with a 0.67% forced-draw spread. The 1,000-row update-only median was **4.4850 ms**; update plus forced draw was **8.6402 ms**. Compared with the recorded A2 values (4.5905 ms and 8.24 ms), update-only is 0.1055 ms faster and forced draw is 0.4002 ms slower. This passes the owner-approved 9.0 ms A3/0.4.0 ceiling; it misses the older 8.0 ms target.
