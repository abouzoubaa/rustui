# F0 decision — GPUI Fast 0.1.14

Decision point: 2026-11-14. This evaluates the candidate on `f0-integration`; it does not switch `main` or authorize adoption. The native checklist in [F0-native-checklist.md](F0-native-checklist.md) has not been run by the owner.

## Decision criteria

| Criterion | Result | Evidence |
|---|---|---|
| Full `scripts/check.sh`: at least 1,826 passed, zero failures | **PASS** | PR #133 source job: 1,441 + 9 + 86 + 40 + 255 = **1,831 passed / 0 failed / 7 ignored**. The seven ignored entries are the existing manual/performance cases; no F0 test was ignored. |
| Every focused matrix and cumulative proof passes | **PASS (automated)** | The Kit 0.9 cumulative fixture applied/reversed 64 files exactly (15 core; A3-4a 3; 4b 10; 4c 8; 4d 6; 4e 8; 4f 2; 4g 1; 4h 4; 4i 2). F0's 18-patch core candidate stack passed exact-hash apply/reverse. PR #133 source job passed. Focused RustUI results are listed below. |
| `scripts/smoke-playground.sh` | **PASS** | Exit 0; the temporary Playground app stayed running for 10 seconds and produced no new crash report. |
| 1,000-row update + forced draw: at least 10% faster than 8.64 ms (target ≤ 7.776 ms) | **PASS** | Three-run median **5.658 ms** (5.595, 5.568, 5.722 ms), versus the recorded gpui-pre baseline **8.790 ms**. Update-only median: 3.363 ms vs 4.678 ms. |
| Long-list scrolling: at least 20% lower median frame time | **PASS** | 5,000-row / 300-frame median **0.630 ms**, p95 **0.740 ms**, versus **11.277 / 12.629 ms** on gpui-pre. Candidate run medians: 0.632, 0.630, 0.622 ms; p95s: 0.712, 0.745, 0.740 ms. Rebuilt rows fell from 40 to 20 per frame. |
| No automated stale-rendering failures and owner native stale-rendering checklist passes | **FAIL — owner check pending** | All five no-forced-refresh GPUI test-window cases passed on both the gpui-pre baseline and the Fast candidate. The owner has not run the native checklist, so this combined criterion is not satisfied. |

## Additional F0 integration gates

| Gate | Result | Evidence |
|---|---|---|
| One GPUI backend in RustUI's macOS target graph | **PASS** | `rustup run 1.98.0 cargo tree -p rustui -i gpui-fast --target aarch64-apple-darwin` exits 0 and shows `gpui-fast v0.1.14` as the core used by RustUI/Base/Component. Filtered metadata reports `gpui-pre=0`, `gpui-fast=1`. |
| Workspace-wide metadata has no `gpui-pre` package | **FAIL** | Unfiltered `cargo metadata --format-version 1 --locked --offline` reports `gpui-pre=2`, `gpui-fast=1`. The two `gpui-pre` records are the owned path crate and registry package retained as dormant workspace/optional records; they are absent from RustUI's filtered macOS graph. The requested unfiltered zero-count proof therefore does not pass. Also, `cargo tree -i gpui-pre` is ambiguous until a source-qualified package ID is supplied. |
| Source checks and owned-source mirror verifier | **PASS** | PR #133 source job passed in 18m42s, including `verify-release.sh source`, candidate fixture checks, and `verify_owned_gpui_sources.py`. |

## Automated focused results

All listed commands were run against the Fast candidate with Rust 1.98.0 and the shared target cache. Test counts are passed / failed / ignored.

| Suite | Result |
|---|---:|
| Disabled-control matrix | 201 / 0 / 0 |
| Async task ownership/cancellation | 19 / 0 / 0 |
| Menu popup lifecycle | 4 / 0 / 0 |
| Popover lifecycle | 7 / 0 / 0 |
| Focus recovery | 11 / 0 / 0 |
| EditableDataGrid | 32 / 0 / 0 |
| Validation descriptions | 4 / 0 / 0 |
| File drop | 11 / 0 / 0 |
| Theme reload | 2 / 0 / 0 |
| Transitions | 6 / 0 / 0 |
| Rust/JSON syntax highlighter | 1 / 0 / 0 |

The five stale-rendering tests, each passed on both `origin/main` (gpui-pre) and the Fast candidate:

- `f0_stale_rendering_owner_environment_list_and_transition_matrix`
- `f0_stale_rendering_theme_change_invalidates_visible_output`
- `f0_stale_rendering_text_input_binding_updates_without_forced_refresh`
- `f0_stale_rendering_grid_edit_updates_cell_without_forced_refresh`
- `f0_stale_rendering_popup_menu_open_and_close_updates_tree`

No test assertion was weakened. The current full check is 1,831/0/7; the historical F0-4 findings of 1,388/48/7 remain recorded in [F0-findings.md](F0-findings.md). The cause of the earlier failure set was not established, so this report does not attribute it to the cache.

## Additional benchmark comparison

All new workloads use the same Apple M1 Max, macOS 27.0, Rust 1.98.0 GPUI test-window methodology and three runs per backend. Values are medians of per-run medians and p95s.

| Workload | gpui-pre baseline median / p95 | Fast candidate median / p95 | Result |
|---|---:|---:|---|
| 5,000-row list scroll | 11.277 / 12.629 ms | 0.630 / 0.740 ms | Faster; 20 rows/frame rebuilt vs 40 |
| Typing in 10,000-line Rust editor | 5.775 / 6.972 ms | 0.699 / 0.902 ms | Faster |
| Idle draw with 60 mounted panels | 3.702 / 4.257 ms | 0.017 / 0.017 ms | Faster; post-change owner render count remains 3 |
| 1,000-row update + forced draw | 8.790 ms | 5.658 ms | Faster |

## Risks and owner decision

- Native/manual acceptance is **not verified**. Run every step in `F0-native-checklist.md`, including window resize/display movement, rapid scroll, editing, and accessibility-tree checks.
- The stricter workspace-wide metadata count remains nonzero even though the RustUI macOS target graph uses only GPUI Fast. Decide whether the dormant package records must be removed before adoption.
- The full automated test count improved relative to the recorded F0-4 run, but the earlier 48 failures' cause was not proven.
- All automated performance gates pass. The two failures above mean the F0 adoption criteria are not all met. Keep `main` on gpui-pre until the owner reviews the native results and decides how to resolve the metadata gate.
