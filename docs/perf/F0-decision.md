# F0 decision — GPUI Fast 0.1.14

Owner decision 2026-11-20: adopt GPUI Fast. Owner native/visual review still pending (scrolling comparison, R-1 grid).

Historical evaluation point: 2026-11-14. The evaluation below did not authorize adoption; the dated owner decision above supersedes that recommendation. The native checklist in [F0-native-checklist.md](F0-native-checklist.md) has not been run by the owner.

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
- All automated performance gates passed at evaluation time. The owner decision above authorizes adoption with the native/visual review still pending; it does not mark that review complete.


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


## Fast adoption source ownership (FA-3)

The macOS build uses the owned Fast core, platform, macOS, Apple and macro packages. The old owned core `crates/rustui-gpui` remains only because the deferred-platform mirrors (`rustui-gpui-linux`, `rustui-gpui-wgpu`, `rustui-gpui-windows`, `rustui-gpui-web`) and optional Kit backend declarations still reference it. Those four manifests and their source files are unchanged. Shared upstream utility packages with `gpui-pre-` names remain dependencies of GPUI Fast itself; they are not the legacy GPUI backend.

| Old owned folders | Disposition | Why |
|---|---|---|
| `rustui-gpui` | Kept outside workspace members | Required by the deferred-platform mirrors and optional Kit declarations; absent from the RustUI macOS dependency closure. |
| `rustui-gpui-platform`, `rustui-gpui-macos`, `rustui-gpui-apple` | Archived under `archive/pre-fast/crates/` | The final live path reference was the assets crate's dev platform, now wired to Fast by patch. |
| `rustui-block`, `rustui-cocoa`, `rustui-cocoa-foundation`, `rustui-metal`, `rustui-objc-foundation` | Archived under `archive/pre-fast/crates/` | Used only by the retired owned macOS chain; no remaining live path dependency selects them. |

The archive patch moves 184 files unchanged. Popup geometry checks now read the active owned Fast macOS source with their assertions unchanged; separate archive tests preserve the historical file images. No deferred-platform dependency declaration was changed. The gpui-pre benchmark baseline is reproduced from the rollback tag, not from live macOS crate selection:

```sh
git worktree add --detach ../rustui-pre-fast archive/pre-fast
cd ../rustui-pre-fast
export CARGO_TARGET_DIR="$HOME/.cache/rustui-target"
. scripts/pinned-toolchain.sh
pinned_cargo test --release -p rustui --lib f0_long_list_scroll_benchmark --locked --offline -- --ignored --nocapture
pinned_cargo test --release -p rustui --lib f0_large_editor_typing_benchmark --locked --offline -- --ignored --nocapture
pinned_cargo test --release -p rustui --lib f0_idle_redraw_invalidation_benchmark --locked --offline -- --ignored --nocapture
pinned_cargo test --release -p rustui --lib owner_update_and_draw_workload --locked --offline -- --ignored --nocapture
```


## Adoption benchmarks (FA-4, 2026-11-20)

The following are five-run warmed GPUI test-window measurements on AC power with Low Power Mode off. The benchmark binary was built with pinned Rust 1.98.0 using `cargo test --release -p rustui --lib --no-run --locked --offline --message-format=json`, in `$HOME/.cache/rustui-target-r1`. Cargo reports opt-level 3, no debug assertions; the repository release profile sets thin LTO and one codegen unit. No profile overrides were set, and no Cargo/rustc build ran during timing. Each filter ran directly in the emitted test executable with `--ignored --nocapture --test-threads=1`; its existing warm-up is unchanged. These CPU/test-window timings are not native display-frame timings.

| Workload | Historical gpui-pre reference median (ms) | Adoption median / p95 (ms) | Five-run spread |
|---|---:|---:|---:|
| 5,000-row scroll | 1.274208 | 0.119584 / 0.143917 | 3.76% |
| 10,000-line editor typing | 0.655584 | 0.556083 / 0.865334 | 3.93% |
| Idle draw, 60 panels | 0.257917 | 0.016667 / 0.017041 | 5.25% |
| 1,000-row update + forced draw | 8.762925 | 5.546938 / 5.924442 | 5.61% |

The reference values are the matched release-profile F0-10 gpui-pre measurements at `archive/pre-fast`; they were not re-run in FA-4. The adoption candidate also includes the later F0 RustUI hardening, so the difference is not attributed solely to the backend swap.

| Workload | Run medians 1–5 (ms) |
|---|---|
| 5,000-row scroll | 0.119584; 0.121500; 0.119333; 0.119583; 0.123833 |
| 10,000-line editor typing | 0.557042; 0.548542; 0.570417; 0.556083; 0.553583 |
| Idle draw, 60 panels | 0.016750; 0.016875; 0.016667; 0.016667; 0.016000 |
| 1,000-row update + forced draw | 5.369846; 5.681237; 5.399500; 5.546938; 5.674646 |

The initial forced-draw set had medians 6.299821, 5.331583, 5.252696, 5.252500, 5.492938 ms (19.64% spread). Per the F0 reliability method, it was repeated once; the repeat set above is used.

Known native issue retained from F0-14: Collection has 1–3 presentation gaps above 16.7 ms per capture run. Scrolling comparison and R-1 grid visual review remain pending; these automated results do not complete that review.


## Collection cell retention

Collection retains cell entities and subscriptions for GPUI’s actual uniform-list visible range plus the pinned Fast overdraw and batched-eviction margin. Freshly prepainted rows are included too. Reconciliation runs after native painting and skips unchanged ranges when no row was laid out; a cached row skipping RustUI’s prepaint hook is never treated as removal. Snapshot reconciliation still removes IDs absent from the items. No GPUI source or public API changes are needed.

The replay fixture uses the production Collection element and retained-cell store with a stable native list, so per-render owner updates cannot hide cache replay. It checks the row-prepaint counter, retained model/subscription, control interaction, and release outside overdraw or after item removal.

Other row-based components do not use Collection’s prepaint-visit eviction:

| Component | Source | Retention strategy |
|---|---|---|
| List | `crates/rustui/src/components/data/list.rs:382,453` | Component-level ListState and retained event subscription; no per-row prepaint pruning. |
| KeyedList | `crates/rustui/src/components/data/keyed_list.rs:552,612` | Component-level ListState/delegate and retained subscription; no per-row prepaint pruning. |
| Table | `crates/rustui/src/components/data/table.rs:505,608,647` | Component-level TableState/subscription. Its prepaint callback aligns the horizontal RTL anchor; it does not evict cells. |
| DataGrid | `crates/rustui/src/components/data/data_grid.rs:96` | Delegates rendering and state to Table. |
| EditableDataGrid | `crates/rustui/src/components/data/editable_data_grid.rs:1121,1274` | Retains GridState, the editor and TableState at component scope; no per-row visit eviction. |
| Tree | `crates/rustui/src/components/data/tree.rs:576,663` | Component-level RetainedTree/TreeState and observer; no per-row prepaint pruning. |
| Outline | `crates/rustui/src/components/data/outline.rs:98` | Delegates to Tree. |
| LazyGrid | `crates/rustui/src/layout/lazy_grid.rs:232` | Prunes against authoritative item IDs, not prepaint visits. |

## Final adoption benchmarks (FA-5)

These measurements include the Collection retention fix. The pinned Rust 1.98.0 release test executable was built in `$HOME/.cache/rustui-target` with `pinned_cargo test --release -p rustui --lib --no-run --locked --offline --message-format=json` (opt-level 3, no debug assertions, thin LTO, one codegen unit). All compilation finished before timing, on AC power with Low Power Mode off. Each workload ran directly in the emitted executable with `--ignored --nocapture --test-threads=1`, using its existing warm-up. Each final aggregate is the median of five run medians; p95 is the median of the five run p95s. These are CPU/test-window timings, not native presentation timings. Historical gpui-pre values are the F0-10 reference, not a new baseline run.

| Workload | Historical gpui-pre median (ms) | Final median / p95 (ms) | Spread |
|---|---:|---:|---:|
| 5,000-row scroll | 1.274208 | 0.128041 / 0.162500 | 2.83% |
| 10,000-line editor typing | 0.655584 | 0.559000 / 0.837625 | 2.80% |
| Idle draw, 60 panels | 0.257917 | 0.016500 / 0.017166 | 4.04% |
| 1,000-row update + forced draw | 8.762925 | 5.493033 / 5.670300 | 6.50% |

| Workload | Run medians 1–5 (ms) |
|---|---|
| 5,000-row scroll | 0.130000; 0.127000; 0.128167; 0.128041; 0.126375 |
| 10,000-line editor typing | 0.555250; 0.563500; 0.565292; 0.549667; 0.559000 |
| Idle draw, 60 panels | 0.016500; 0.016542; 0.016291; 0.016791; 0.016125 |
| 1,000-row update + forced draw | 5.670604; 5.493033; 5.392867; 5.524438; 5.313317 |

The initial forced-draw set had medians 5.685050, 5.429021, 6.222846, 5.468975, 5.520313 ms (14.38% spread). It was repeated once; the final set above is used.
