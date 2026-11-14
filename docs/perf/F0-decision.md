# F0 decision — GPUI Fast 0.1.14

Decision point: 2026-11-14. This evaluates the candidate on `f0-integration`; it does not switch `main` or authorize adoption. The native checklist in [F0-native-checklist.md](F0-native-checklist.md) has not been run by the owner.

## Decision criteria

| Criterion | Result | Evidence |
|---|---|---|
| Full `scripts/check.sh`: at least 1,826 passed, zero failures | **PASS** | PR #133 source job: 1,441 + 9 + 86 + 40 + 255 = **1,831 passed / 0 failed / 7 ignored**. The seven ignored entries are the existing manual/performance cases; no F0 test was ignored. |
| Every focused matrix and cumulative proof passes | **PASS (automated)** | The Kit 0.9 cumulative fixture applied/reversed 64 files exactly (15 core; A3-4a 3; 4b 10; 4c 8; 4d 6; 4e 8; 4f 2; 4g 1; 4h 4; 4i 2). F0's 18-patch core candidate stack passed exact-hash apply/reverse. PR #133 source job passed. Focused RustUI results are listed below. |
| `scripts/smoke-playground.sh` | **PASS** | Exit 0; the temporary Playground app stayed running for 10 seconds and produced no new crash report. |
| 1,000-row update + forced draw: at least 10% faster than 8.64 ms (target ≤ 7.776 ms) | **PASS** | F0-10 release median **5.663 ms** (5.855, 5.663, 5.643, 5.743, 5.495 ms), versus same-profile gpui-pre **8.763 ms**; spread 6.35% Fast / 1.07% baseline. |
| Long-list scrolling: at least 20% lower median frame time | **PASS** | F0-10 release median **0.622 ms**, p95 **0.776 ms**, versus same-profile gpui-pre **1.274 / 1.513 ms**; spread 1.78% Fast / 2.88% baseline. Rebuilt rows fell from 40 to 20 per frame. |
| No automated stale-rendering failures and owner native stale-rendering checklist passes | **FAIL — owner check pending** | All five no-forced-refresh GPUI test-window cases passed on both the gpui-pre baseline and the Fast candidate. The owner has not run the native checklist, so this combined criterion is not satisfied. |

## Additional F0 integration gates

| Gate | Result | Evidence |
|---|---|---|
| One GPUI backend in RustUI's macOS target graph | **PASS** | `rustup run 1.98.0 cargo tree -p rustui -i gpui-fast --target aarch64-apple-darwin` exits 0 and shows `gpui-fast v0.1.14` as the core used by RustUI/Base/Component. Filtered metadata reports `gpui-pre=0`, `gpui-fast=1`. |
| Workspace-wide metadata has no `gpui-pre` package | **PENDING OWNER CONFIRMATION** | **Accepted by design: gpui-pre only in deferred-platform and optional Kit records; RustUI macOS graph has one GPUI.** Unfiltered metadata retains two records (`gpui-pre=2`, `gpui-fast=1`); the filtered RustUI macOS graph has only GPUI Fast. |
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

F0-10's official values use the same Apple M1 Max, macOS 27.0, Rust 1.98.0 GPUI test-window methodology and five interleaved runs per backend. Values are medians of per-run medians and p95s; earlier run counts are historical only.

| Workload | gpui-pre baseline median / p95 | Fast candidate median / p95 | Result |
|---|---:|---:|---|
| 5,000-row list scroll | 1.274 / 1.513 ms | 0.622 / 0.776 ms | F0-10 official aligned release comparison; 20 rows/frame rebuilt vs 40 |
| Typing in 10,000-line Rust editor | 0.656 / 0.980 ms | 0.633 / 0.957 ms | F0-10 official aligned release comparison |
| Idle draw with 60 mounted panels | 0.258 / 0.334 ms | 0.016 / 0.018 ms | F0-10 official aligned release comparison; 63 vs 2 idle owner renders |
| 1,000-row update + forced draw | 8.763 / 9.034 ms | 5.663 / 6.097 ms | F0-10 official aligned release comparison |

The official values above are five-run interleaved release-profile comparisons. Full per-run results, profiles, warm-up details, and the historical F0-8/F0-9 method discrepancy are documented in [F0-bench-method.md](F0-bench-method.md). The earlier F0-8 and F0-9 measurements remain below as historical data; they are not the official aligned comparison.

## Risks and owner decision

- Native/manual acceptance is **not verified**. Run every step in `F0-native-checklist.md`, including window resize/display movement, rapid scroll, editing, and accessibility-tree checks.
- Workspace metadata is accepted by design as stated above, pending owner confirmation.
- The full automated test count improved relative to the recorded F0-4 run, but the earlier 48 failures' cause was not proven.
- All automated performance gates pass. The owner has not run the native stale-rendering checklist; adoption remains unapproved pending that native result and confirmation of the metadata-gate wording.


## Changes since first decision (F0-9)

- Missing-notify audit: `docs/perf/F0-notify-audit.md` records the production component/view entity updates and notification path for each. It found no RustUI production update that changes visible output without an invalidation; no production notify fix was needed. The existing five no-forced-refresh tests remain the behavioral coverage.
- NaN content mask: resolved in RustUI GraphView by not creating absolute node elements for projected rectangles outside the viewport. The offending node was `rustui-graph-node ... b` in the GraphView GPUI test: a finite ~`3.06e38px` x coordinate / `3.40e36px` width was outside the 600×400 viewport but still lowered to a Div. The original GPUI Fast `debug_assert_eq!` test passes after the culling change. The NaN-equality patch was removed.
- Dormant metadata records: fresh unfiltered `cargo metadata --format-version 1 --locked --offline` reports `gpui-pre=2` and `gpui-fast=1`. The path `gpui-pre 0.3.12` record is referenced by optional `gpui` dependencies in owned `gpui-base`, `gpui-component`, and `gpui-kit-assets`, by the `gpui-component` dev dependency, and by `gpui-pre-web` / `gpui-pre-wgpu`. The registry `gpui-pre 0.3.12` record is referenced by upstream `gpui-pre-apple` and `gpui-pre-ios`. Those package-level references preserve gpui-pre comparison/dev and deferred-platform package graphs; neither `gpui-pre` record is in RustUI’s filtered macOS target graph. They were left intact in F0-9.

### Fresh F0-9 verification

| Command | Result |
|---|---|
| `scripts/check.sh` | Exit 0; 1,831 passed / 0 failed / 7 ignored (groups: 1,441; 9; 86; 40; 255; 0) |
| `./scripts/verify-release.sh source` | Exit 0; Fast core candidate stack 17 patches, exact hashes PASS; approved source patch gates PASS |
| `scripts/smoke-playground.sh` | Exit 0; temporary Playground app stayed running for 10 seconds with no new crash report |
| Focused graph regression, original GPUI Fast assertion | `rendered_node_editor_connects_moves_and_deletes_nodes`: 1 passed / 0 failed / 0 ignored; also passed in full check |
| `cargo metadata --format-version 1 --locked --offline` | Exit 0; `gpui-pre=2`, `gpui-fast=1`; reasons listed above |

The first three F0 workload sets were discarded because they overlapped the release-mode 1,000-row benchmark build. These replacement measurements ran with no concurrent cargo build, three runs each. The existing release benchmark also ran three times.

| Workload | gpui-pre baseline median / p95 | First Fast decision median / p95 | F0-9 fresh median / p95 | Notes |
|---|---:|---:|---:|---|
| 5,000-row list scroll | 11.277 / 12.629 ms | 0.630 / 0.740 ms | 5.586 / 6.290 ms | Still faster than gpui-pre; 20 rows/frame rebuilt (6,000 / 300 frames) |
| Typing in 10,000-line Rust editor | 5.775 / 6.972 ms | 0.699 / 0.902 ms | 5.068 / 6.012 ms | Faster than gpui-pre in this run |
| Idle draw with 60 mounted panels | 3.702 / 4.257 ms | 0.017 / 0.017 ms | 0.183 / 0.207 ms | Faster than gpui-pre; 2 idle body renders, 3 after owner update |
| 1,000-row update + forced draw | 8.790 ms | 5.658 ms | 5.535 ms | Release harness: 5.478, 5.668, 5.543 ms; 3.42% spread |

The first three fresh workload timings are materially higher than the first Fast measurements, although still below the recorded gpui-pre medians. This run does not establish that the F0-9 GraphView culling change caused the difference; the measured list/editor/idle workloads do not exercise GraphView. Keep this measurement variance visible for the owner review.

The F0-9 notify audit found no RustUI production component/view update with a missing invalidation, so no new notify-specific test was required. The existing five no-forced-refresh tests remain part of the 1,441-test RustUI group; the GraphView regression above exercises the offscreen extreme-node fix.
