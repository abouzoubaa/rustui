# P0 — damage-off CPU profiling

P0 only, as the owner-approved exception between G2-8a and G2-8b. **No runtime code, Cargo configuration, lockfile, vendor source or patch changed.** The PR contains this report and a changelog line. P1 and G2-8b are not started.

## Profiled source and method

- Profiled commit: **`978c67de8c5ce953510d83fde9e8d6cf70fdab6f`**, the current `g-integration` head when P0 began. Local and remote `p-integration` already pointed to exactly that commit; they were reused without resetting either branch. Profiling work used an isolated `p0-profiling` worktree.
- Host: MacBookPro18,2, Apple M1 Max (8 performance + 2 efficiency cores), 64 GB RAM, macOS 27.0 build 26A5425a, AC power.
- Pinned Rust **1.98.0**, through `scripts/pinned-toolchain.sh`. A separate target directory kept this build independent of G2-7/G2-8 artifacts. Symbols were configured only through the requested environment; `Cargo.toml` was unchanged.

```sh
source scripts/pinned-toolchain.sh
export CARGO_TARGET_DIR="$HOME/.cache/rustui-p0-20261121-target"
CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_STRIP=none \
  pinned_cargo build -p rustui-lab --release --locked --offline
RUSTUI_DAMAGE=0 RUSTUI_FRAME_LOG=1 RUSTUI_FRAME_PROFILE=1 \
  samply record --save-only --unstable-presymbolicate \
  --output target/headstart/performance/p0/log-1.json.gz \
  "$CARGO_TARGET_DIR/release/rustui-lab" --g0-scene log --duration 60
```

Samply 0.13.1 used its default 1,000 Hz rate and all target-process threads. It recorded the app from launch; analysis trimmed strictly to the existing `RUSTUI_G0_LOG start/end` timestamps **after normal warmup**. Each required log run validated 600 appends. Typing, hover, drag and idle were then recorded once each, in that order; no F0 acceptance panels ran. A separate log control used `RUSTUI_FRAME_PROFILE=0` while retaining `RUSTUI_FRAME_LOG=1`. **`RUSTUI_DAMAGE=0` throughout**, including the allocation attempt. No compilation, other profiling or analysis overlapped the Samply recordings. The executable passed its before/after hash check.

The local throwaway analyzer is `target/headstart/p0/analyze.py`; captures, symbol sidecars and numeric analysis are under `target/headstart/performance/p0/`. Raw profiles and logs are not committed. It resolves sidecar symbols, clips each sample interval to the measured window, weights by `threadCPUDelta` (microseconds), and divides summed process CPU by timed `present` records. Thus **“frame” means attempted present**, not an input event or necessarily a new prepaint. Inclusive times deduplicate recursive occurrences of the same symbol; they overlap and must not be added together. **100% means all sampled process CPU per attempted present**, not the display interval.

Crate totals use the innermost available inline source-frame owner and are disjoint. The requested five groups are retained, with explicit Lab, other-dependency and unresolved-owner residuals. In particular, Tree-sitter C CPU is an external dependency, not literal Kit self time; the Kit caller's inclusive time shows who causes it. Function tables use **canonical physical symbols**, charging inlined code to its containing symbol. This avoids merging unrelated bare DWARF names such as `clone`, `from` or `{closure#0}`. Source points are sampled lines where available, not asserted function-entry lines. Unresolved native addresses remain in raw rankings and totals; extra named-function tables make the callers readable. A recorded zero means no self samples, not proof of zero cost.

`RUSTUI_FRAME_LOG cpu_ms` is **elapsed `Instant` time**, despite its name. Samply CPU excludes off-CPU waits/preemption; phase elapsed time includes them. Main-thread samples inside recorded prepaint intervals provide the prepaint breakdown. Small boundary interpolation and sampling error remain. The analyzer verifies that exclusive owner-group CPU sums to total CPU.

## Capture summary

| Capture | Seconds after warmup | Attempts | Prepaint calls | Process CPU ms/frame | Main CPU ms/frame | Prepaint elapsed ms/call | Unresolved leaf names % |
|---|---:|---:|---:|---:|---:|---:|---:|
| log-1 | 60.100957 | 576 | 576 | 7.628879 | 6.817030 | 5.177368 | 3.184 |
| log-2 | 60.009279 | 575 | 575 | 7.735846 | 6.922879 | 5.188527 | 3.321 |
| typing-1 | 60.080651 | 519 | 599 | 14.288096 | 13.404252 | 0.735691 | 3.159 |
| hover-1 | 60.006285 | 7185 | 5408 | 2.839364 | 2.441889 | 2.031545 | 3.532 |
| drag-1 | 60.006610 | 7188 | 5852 | 2.361954 | 1.652244 | 0.289644 | 4.116 |
| idle-1 | 60.150190 | 114 | 114 | 4.094373 | 3.088884 | 0.474122 | 3.170 |

## Log: both runs and ranking stability

| Self-CPU owner | log-1 ms/frame / % | log-2 ms/frame / % |
|---|---:|---:|
| RustUI-owned | 0.000000 / 0.000 | 0.000000 / 0.000 |
| GPUI Kit | 0.008311 / 0.109 | 0.005970 / 0.077 |
| GPUI Fast | 0.414774 / 5.437 | 0.368277 / 4.761 |
| std/allocator | 0.322517 / 4.228 | 0.320052 / 4.137 |
| macOS system | 6.736563 / 88.303 | 6.904861 / 89.258 |
| Lab workload | 0.000021 / 0.000 | 0.000000 / 0.000 |
| Other dependencies | 0.135319 / 1.774 | 0.133866 / 1.730 |
| Unresolved | 0.011373 / 0.149 | 0.002819 / 0.036 |

| Comparison | Result |
|---|---|
| Named inclusive top-20 / top-5 overlap | 20/20; 5/5 |
| Named self top-20 / top-5 overlap | 14/20; 4/5 |

The dominant log bottleneck is stable: `write` is self rank 1 in both runs; `gpui::window::f0_frame_record` is 4.872290 / 4.999760 inclusive ms/frame (63.866% / 64.631% of process CPU). The low-cost self-time tail reorders: 14/20 overlap does **not** justify a precise ordering of those micro-costs. Inclusive root/event-loop wrappers are attribution containers, not independent optimization targets.

### Log prepaint: what makes up the historical ~5.3 ms

| Component | Run 1 ms/frame | Run 2 ms/frame | Interpretation |
|---|---:|---:|---|
| phase recorder, including formatting and writes | 3.608035 | 3.683934 | Nested layout record formatting and stderr writes, not the layout algorithm. |
| layout engine, excluding recorder | 0.096523 | 0.104567 | Layout computation after excluding recorder stacks. |
| uniform-list rows, excluding layout/recorder | 0.525007 | 0.491761 | Other row/layout-list bookkeeping and child work. |
| other prepaint | 0.280132 | 0.258923 | Remaining prepaint CPU, including system/dispatch work. |
| **Sampled main-thread prepaint CPU** | **4.509697** | **4.539186** | Disjoint buckets above. |
| **Recorded prepaint elapsed time** | **5.177368** | **5.188527** | Remaining elapsed-minus-CPU time is not attributed to a function. |

At [window.rs:5792–5798](../../crates/rustui-gpui-fast/src/window.rs#L5792), every `compute_layout` call records a phase through [f0_frame_record:1384](../../crates/rustui-gpui-fast/src/window.rs#L1384). Those writes happen *inside* the outer prepaint interval. There are **19,645 / 19,611 layout records**, or **34.105903 / 34.106087 per frame**. Mean recorded layout computation is only 0.003407 / 0.003282 ms/call; the logger is outside that inner timer but inside prepaint. Nested logging accounts for 3.608035 / 3.683934 ms/frame of sampled prepaint CPU. Kernel `write` alone accounts for 3.551079 / 3.633736 ms/frame inside prepaint.

The 800-point scene has 32-point padding and 24-point rows ([g0.rs](../../apps/rustui-lab/src/g0.rs#L188)); the nominal 736-point viewport supports about **31 visible rows**, with partial-row/layout measurement effects. This is source-derived geometry, not a measured render-range counter. Normalizing by 31 gives approximately 0.1164 / 0.1188 ms per visible row for nested logging, versus 0.0031 / 0.0034 ms per row for sampled layout computation excluding logging. These are normalization estimates, not measured row invocation times.

### Observer-overhead control (one diagnostic capture)

With only `RUSTUI_FRAME_PROFILE=0`, log completed 600 appends and 575 presents. Process CPU was **3.436339 ms/frame**, versus 7.628879 / 7.735846 in the two phase-enabled runs. Recorder inclusive CPU fell to **0.262706 ms/frame**; draw/present and GPU logging remained enabled. This supports a substantial phase-logging observer effect. It is **not** a matched 5×5 result or a measured saving from a code fix. No prepaint timer exists in the control, so it cannot supply a replacement elapsed prepaint figure.

### RustUI-owned code in the log

Neither required log run contains a RustUI-owned physical self sample or named RustUI-owned hot function. The log rows come directly from the Lab's native GPUI `UniformList`, while the parent RustUI view is retained. No claim is made that production RustUI log components have zero cost. The workload's [NativeSurface::render](../../apps/rustui-lab/src/g0.rs#L183) clones its whole `Vec<SharedString>` once per render and collects row `Div`s; [append](../../apps/rustui-lab/src/g0.rs#L151) formats one new line per append. These are **Lab workload**, not `crates/rustui` fixes, and neither explains the measured 3.6 ms prepaint logger subtree.

### Log function tables


#### log-1 — top 20 inclusive, unpruned

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `UNRESOLVED HIToolbox+0xbb58f` | macOS system | 6.817030 | 89.358 | — |
| `UNRESOLVED HIToolbox+0xbe91b` | macOS system | 6.817030 | 89.358 | — |
| `UNRESOLVED HIToolbox+0x24137f` | macOS system | 6.817030 | 89.358 | — |
| `UNRESOLVED AppKit+0x37c28f` | macOS system | 6.817030 | 89.358 | — |
| `UNRESOLVED AppKit+0x394fb` | macOS system | 6.817030 | 89.358 | — |
| `UNRESOLVED AppKit+0x8b16ff` | macOS system | 6.817030 | 89.358 | — |
| `UNRESOLVED AppKit+0x8b1223` | macOS system | 6.817030 | 89.358 | — |
| `UNRESOLVED AppKit+0x2c90b` | macOS system | 6.817030 | 89.358 | — |
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 6.817030 | 89.358 | [crates/rustui-gpui-fast-macos/src/platform.rs:563](../../crates/rustui-gpui-fast-macos/src/platform.rs#L563) |
| `rustui_lab::g0::run_cli` | Lab workload | 6.817030 | 89.358 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 6.817030 | 89.358 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 6.817030 | 89.358 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 6.817030 | 89.358 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 6.817030 | 89.358 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 6.817030 | 89.358 | — |
| `start` | macOS system | 6.817030 | 89.358 | — |
| `UNRESOLVED CoreFoundation+0x15a643` | macOS system | 6.815051 | 89.332 | — |
| `UNRESOLVED CoreFoundation+0x81e0b` | macOS system | 5.899370 | 77.329 | — |
| `UNRESOLVED CoreFoundation+0x81d33` | macOS system | 5.899370 | 77.329 | — |
| `UNRESOLVED CoreFoundation+0x8062b` | macOS system | 5.899370 | 77.329 | — |

#### log-1 — top 20 named inclusive functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 6.817030 | 89.358 | [crates/rustui-gpui-fast-macos/src/platform.rs:563](../../crates/rustui-gpui-fast-macos/src/platform.rs#L563) |
| `rustui_lab::g0::run_cli` | Lab workload | 6.817030 | 89.358 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 6.817030 | 89.358 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 6.817030 | 89.358 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 6.817030 | 89.358 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 6.817030 | 89.358 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 6.817030 | 89.358 | — |
| `start` | macOS system | 6.817030 | 89.358 | — |
| `<gpui::app::App>::update_window_erased` | GPUI Fast | 5.887833 | 77.178 | [crates/rustui-gpui-fast/src/app.rs:3268](../../crates/rustui-gpui-fast/src/app.rs#L3268) |
| `<gpui_macos::window::MacWindowState>::start_display_link::{closure#0}` | GPUI Fast | 5.887658 | 77.176 | [crates/rustui-gpui-fast-macos/src/window.rs:858](../../crates/rustui-gpui-fast-macos/src/window.rs#L858) |
| `<gpui_apple::display_link::MetalDisplayLinkDelegate>::needs_update` | GPUI Fast | 5.887658 | 77.176 | [crates/rustui-gpui-fast-apple/src/display_link.rs:29](../../crates/rustui-gpui-fast-apple/src/display_link.rs#L29) |
| `<gpui::window::Window>::new::{closure#10}` | GPUI Fast | 5.887587 | 77.175 | [crates/rustui-gpui-fast/src/window.rs:1999](../../crates/rustui-gpui-fast/src/window.rs#L1999) |
| `<gpui::app::App>::update_window_id::<(), <gpui::window::Window>::new::{closure#10}::{closure#6}::{closure#0}>::{closure#0}` | GPUI Fast | 5.887441 | 77.173 | [crates/rustui-gpui-fast/src/app.rs:2368](../../crates/rustui-gpui-fast/src/app.rs#L2368) |
| `<gpui::window::Window>::new::{closure#10}::{closure#6}` | GPUI Fast | 5.887441 | 77.173 | [crates/rustui-gpui-fast/src/window.rs:2102](../../crates/rustui-gpui-fast/src/window.rs#L2102) |
| `<gpui::window::Window>::draw` | GPUI Fast | 5.667691 | 74.293 | [crates/rustui-gpui-fast/src/window.rs:3944](../../crates/rustui-gpui-fast/src/window.rs#L3944) |
| `<gpui::window::Window>::draw_roots` | GPUI Fast | 5.438436 | 71.287 | [crates/rustui-gpui-fast/src/window.rs:4161](../../crates/rustui-gpui-fast/src/window.rs#L4161) |
| `gpui::window::f0_frame_record` | GPUI Fast | 4.872290 | 63.866 | — |
| `std::io::stdio::_eprint` | std/allocator | 4.861828 | 63.729 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/io/stdio.rs:1284` |
| `<&std::io::stdio::Stderr as std::io::Write>::write_fmt` | std/allocator | 4.859830 | 63.703 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/io/stdio.rs:1067` |
| `core::fmt::write` | std/allocator | 4.857811 | 63.677 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/fmt/mod.rs:1631` |

#### log-1 — top 20 self-time functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `write` | macOS system | 4.787995 | 62.761 | — |
| `mach_msg2_trap` | macOS system | 0.918115 | 12.035 | — |
| `__workq_kernreturn` | macOS system | 0.795243 | 10.424 | — |
| `gpui::fast::layers::tiles::hash_primitive` | GPUI Fast | 0.069960 | 0.917 | [crates/rustui-gpui-fast/src/fast/layers/tiles.rs:?](../../crates/rustui-gpui-fast/src/fast/layers/tiles.rs) |
| `_platform_memmove` | macOS system | 0.043885 | 0.575 | — |
| `<gpui::scene::Scene>::insert_primitive::<gpui::scene::Primitive>` | GPUI Fast | 0.027590 | 0.362 | [crates/rustui-gpui-fast/src/scene.rs:96](../../crates/rustui-gpui-fast/src/scene.rs#L96) |
| `kevent_id` | macOS system | 0.023606 | 0.309 | — |
| `<gpui::window::Window>::draw` | GPUI Fast | 0.020606 | 0.270 | [crates/rustui-gpui-fast/src/window.rs:3944](../../crates/rustui-gpui-fast/src/window.rs#L3944) |
| `gpui::fast::layers::lists::drawn_alike::hash` | GPUI Fast | 0.020354 | 0.267 | [crates/rustui-gpui-fast/src/fast/layers/lists.rs:1644](../../crates/rustui-gpui-fast/src/fast/layers/lists.rs#L1644) |
| `gpui::fast::layers::lists::end_paint_rows` | GPUI Fast | 0.018948 | 0.248 | [crates/rustui-gpui-fast/src/fast/layers/lists.rs:2706](../../crates/rustui-gpui-fast/src/fast/layers/lists.rs#L2706) |
| `<std::io::default_write_fmt::Adapter<std::io::stdio::StderrLock> as core::fmt::Write>::write_str` | std/allocator | 0.018288 | 0.240 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/io/mod.rs:614` |
| `<gpui::scene::Scene>::insert_primitive::<gpui::scene::MonochromeSprite>` | GPUI Fast | 0.017550 | 0.230 | [crates/rustui-gpui-fast/src/scene.rs:94](../../crates/rustui-gpui-fast/src/scene.rs#L94) |
| `<gpui::taffy::TaffyLayoutEngine>::request_retained_carried_measured_layout::<gpui::fast::text::TextMeasureInputs, gpui::fast::text::adopt_measurement, gpui::fast::text::forget_measurement, gpui::fast::text::measure_text>` | GPUI Fast | 0.016714 | 0.219 | [crates/rustui-gpui-fast/src/fast/layout.rs:741](../../crates/rustui-gpui-fast/src/fast/layout.rs#L741) |
| `gpui::fast::layers::tiles::hash_clipped` | GPUI Fast | 0.016594 | 0.218 | [crates/rustui-gpui-fast/src/fast/layers/tiles.rs:325](../../crates/rustui-gpui-fast/src/fast/layers/tiles.rs#L325) |
| `<gpui::taffy::TaffyLayoutEngine>::layout_bounds` | GPUI Fast | 0.015757 | 0.207 | — |
| `<gpui::taffy::TaffyLayoutEngine>::compute_layout` | GPUI Fast | 0.015738 | 0.206 | — |
| `core::num::imp::flt2dec::strategy::dragon::format_exact` | std/allocator | 0.013969 | 0.183 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/num/imp/flt2dec/strategy/dragon.rs:282` |
| `<gpui::bounds_tree::Grid<gpui::geometry::ScaledPixels>>::add` | GPUI Fast | 0.013569 | 0.178 | [crates/rustui-gpui-fast/src/fast/bounds_tree.rs:340](../../crates/rustui-gpui-fast/src/fast/bounds_tree.rs#L340) |
| `<gpui::style::Style as refineable::Refineable>::refine` | GPUI Fast | 0.013139 | 0.172 | [crates/rustui-gpui-fast/src/style.rs:179](../../crates/rustui-gpui-fast/src/style.rs#L179) |
| `<gpui::bounds_tree::BoundsTree<gpui::geometry::ScaledPixels>>::insert` | GPUI Fast | 0.012227 | 0.160 | — |

#### log-2 — top 20 inclusive, unpruned

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `UNRESOLVED HIToolbox+0x24137f` | macOS system | 6.922879 | 89.491 | — |
| `UNRESOLVED AppKit+0x37c28f` | macOS system | 6.922879 | 89.491 | — |
| `UNRESOLVED AppKit+0x394fb` | macOS system | 6.922879 | 89.491 | — |
| `UNRESOLVED AppKit+0x8b16ff` | macOS system | 6.922879 | 89.491 | — |
| `UNRESOLVED AppKit+0x8b1223` | macOS system | 6.922879 | 89.491 | — |
| `UNRESOLVED AppKit+0x2c90b` | macOS system | 6.922879 | 89.491 | — |
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 6.922879 | 89.491 | [crates/rustui-gpui-fast-macos/src/platform.rs:563](../../crates/rustui-gpui-fast-macos/src/platform.rs#L563) |
| `rustui_lab::g0::run_cli` | Lab workload | 6.922879 | 89.491 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 6.922879 | 89.491 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 6.922879 | 89.491 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 6.922879 | 89.491 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 6.922879 | 89.491 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 6.922879 | 89.491 | — |
| `start` | macOS system | 6.922879 | 89.491 | — |
| `UNRESOLVED HIToolbox+0xbe91b` | macOS system | 6.921374 | 89.471 | — |
| `UNRESOLVED HIToolbox+0xbb58f` | macOS system | 6.919557 | 89.448 | — |
| `UNRESOLVED CoreFoundation+0x15a643` | macOS system | 6.919117 | 89.442 | — |
| `UNRESOLVED QuartzCore+0x6275f` | macOS system | 6.017529 | 77.788 | — |
| `UNRESOLVED CoreFoundation+0xad03f` | macOS system | 6.017529 | 77.788 | — |
| `UNRESOLVED CoreFoundation+0x81e0b` | macOS system | 6.017529 | 77.788 | — |

#### log-2 — top 20 named inclusive functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 6.922879 | 89.491 | [crates/rustui-gpui-fast-macos/src/platform.rs:563](../../crates/rustui-gpui-fast-macos/src/platform.rs#L563) |
| `rustui_lab::g0::run_cli` | Lab workload | 6.922879 | 89.491 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 6.922879 | 89.491 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 6.922879 | 89.491 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 6.922879 | 89.491 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 6.922879 | 89.491 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 6.922879 | 89.491 | — |
| `start` | macOS system | 6.922879 | 89.491 | — |
| `<gpui_apple::display_link::MetalDisplayLinkDelegate>::needs_update` | GPUI Fast | 5.996489 | 77.516 | [crates/rustui-gpui-fast-apple/src/display_link.rs:28](../../crates/rustui-gpui-fast-apple/src/display_link.rs#L28) |
| `<gpui_macos::window::MacWindowState>::start_display_link::{closure#0}` | GPUI Fast | 5.996412 | 77.515 | [crates/rustui-gpui-fast-macos/src/window.rs:856](../../crates/rustui-gpui-fast-macos/src/window.rs#L856) |
| `<gpui::window::Window>::new::{closure#10}` | GPUI Fast | 5.993890 | 77.482 | [crates/rustui-gpui-fast/src/window.rs:?](../../crates/rustui-gpui-fast/src/window.rs) |
| `<gpui::app::App>::update_window_erased` | GPUI Fast | 5.992828 | 77.468 | [crates/rustui-gpui-fast/src/app.rs:3273](../../crates/rustui-gpui-fast/src/app.rs#L3273) |
| `<gpui::app::App>::update_window_id::<(), <gpui::window::Window>::new::{closure#10}::{closure#6}::{closure#0}>::{closure#0}` | GPUI Fast | 5.985962 | 77.380 | [crates/rustui-gpui-fast/src/app.rs:2368](../../crates/rustui-gpui-fast/src/app.rs#L2368) |
| `<gpui::window::Window>::new::{closure#10}::{closure#6}` | GPUI Fast | 5.985962 | 77.380 | [crates/rustui-gpui-fast/src/window.rs:2102](../../crates/rustui-gpui-fast/src/window.rs#L2102) |
| `<gpui::window::Window>::draw` | GPUI Fast | 5.799372 | 74.968 | [crates/rustui-gpui-fast/src/window.rs:3920](../../crates/rustui-gpui-fast/src/window.rs#L3920) |
| `<gpui::window::Window>::draw_roots` | GPUI Fast | 5.588976 | 72.248 | [crates/rustui-gpui-fast/src/window.rs:4161](../../crates/rustui-gpui-fast/src/window.rs#L4161) |
| `gpui::window::f0_frame_record` | GPUI Fast | 4.999760 | 64.631 | — |
| `<&std::io::stdio::Stderr as std::io::Write>::write_fmt` | std/allocator | 4.989330 | 64.496 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/io/stdio.rs:1067` |
| `std::io::stdio::_eprint` | std/allocator | 4.989330 | 64.496 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/io/stdio.rs:1284` |
| `core::fmt::write` | std/allocator | 4.985779 | 64.450 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/fmt/mod.rs:1687` |

#### log-2 — top 20 self-time functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `write` | macOS system | 4.940134 | 63.860 | — |
| `mach_msg2_trap` | macOS system | 0.900378 | 11.639 | — |
| `__workq_kernreturn` | macOS system | 0.797565 | 10.310 | — |
| `gpui::fast::layers::tiles::hash_primitive` | GPUI Fast | 0.078555 | 1.015 | [crates/rustui-gpui-fast/src/fast/layers/tiles.rs:?](../../crates/rustui-gpui-fast/src/fast/layers/tiles.rs) |
| `<gpui::taffy::TaffyLayoutEngine>::request_retained_carried_measured_layout::<gpui::fast::text::TextMeasureInputs, gpui::fast::text::adopt_measurement, gpui::fast::text::forget_measurement, gpui::fast::text::measure_text>` | GPUI Fast | 0.045918 | 0.594 | [crates/rustui-gpui-fast/src/fast/layout.rs:?](../../crates/rustui-gpui-fast/src/fast/layout.rs) |
| `_platform_memmove` | macOS system | 0.044882 | 0.580 | — |
| `<gpui::scene::Scene>::insert_primitive::<gpui::scene::MonochromeSprite>` | GPUI Fast | 0.033322 | 0.431 | [crates/rustui-gpui-fast/src/scene.rs:94](../../crates/rustui-gpui-fast/src/scene.rs#L94) |
| `kevent_id` | macOS system | 0.028489 | 0.368 | — |
| `<gpui::scene::Scene>::insert_primitive::<gpui::scene::Primitive>` | GPUI Fast | 0.024697 | 0.319 | [crates/rustui-gpui-fast/src/scene.rs:96](../../crates/rustui-gpui-fast/src/scene.rs#L96) |
| `gpui::text_system::line::paint_line` | GPUI Fast | 0.023437 | 0.303 | [crates/rustui-gpui-fast/src/text_system/line.rs:585](../../crates/rustui-gpui-fast/src/text_system/line.rs#L585) |
| `gpui::fast::layers::lists::end_paint_rows` | GPUI Fast | 0.019656 | 0.254 | [crates/rustui-gpui-fast/src/fast/layers/lists.rs:2766](../../crates/rustui-gpui-fast/src/fast/layers/lists.rs#L2766) |
| `gpui::fast::layers::lists::drawn_alike::hash` | GPUI Fast | 0.016850 | 0.218 | [crates/rustui-gpui-fast/src/fast/layers/lists.rs:1644](../../crates/rustui-gpui-fast/src/fast/layers/lists.rs#L1644) |
| `UNRESOLVED libsystem_malloc.dylib+0x33a48` | std/allocator | 0.014911 | 0.193 | — |
| `core::slice::sort::unstable::quicksort::quicksort::<u128, <u128 as core::cmp::PartialOrd>::lt>` | std/allocator | 0.013162 | 0.170 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/slice/sort/unstable/quicksort.rs:31` |
| `gpui::fast::layers::tiles::part_tile_hashes` | GPUI Fast | 0.012071 | 0.156 | [crates/rustui-gpui-fast/src/fast/layers/tiles.rs:107](../../crates/rustui-gpui-fast/src/fast/layers/tiles.rs#L107) |
| `UNRESOLVED CoreText+0x1f3c0` | macOS system | 0.011610 | 0.150 | — |
| `<gpui::scene::Primitive as core::clone::Clone>::clone` | GPUI Fast | 0.011188 | 0.145 | [crates/rustui-gpui-fast/src/scene.rs:221](../../crates/rustui-gpui-fast/src/scene.rs#L221) |
| `core::num::imp::flt2dec::strategy::grisu::format_exact_opt` | std/allocator | 0.010010 | 0.129 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/num/imp/flt2dec/strategy/grisu.rs` |
| `<gpui::window::Window>::draw` | GPUI Fast | 0.009630 | 0.124 | [crates/rustui-gpui-fast/src/window.rs:3920](../../crates/rustui-gpui-fast/src/window.rs#L3920) |
| `<hashbrown::map::HashMap<(gpui::element::GlobalElementId, core::any::TypeId), gpui::window::ElementStateBox, rustc_hash::FxBuildHasher>>::remove_entry::<(gpui::element::GlobalElementId, core::any::TypeId)>` | Other dependencies | 0.009336 | 0.121 | `/rust/deps/hashbrown-0.17.1/src/map.rs:1997` |

## Other scenes

| Self-CPU owner | typing-1 ms/frame / % | hover-1 ms/frame / % | drag-1 ms/frame / % | idle-1 ms/frame / % |
|---|---:|---:|---:|---:|
| RustUI-owned | 0.002647 / 0.019 | 0.002240 / 0.079 | 0.000000 / 0.000 | 0.015158 / 0.370 |
| GPUI Kit | 0.081243 / 0.569 | 0.011739 / 0.413 | 0.000534 / 0.023 | 0.018009 / 0.440 |
| GPUI Fast | 0.226775 / 1.587 | 0.152851 / 5.383 | 0.053837 / 2.279 | 0.089053 / 2.175 |
| std/allocator | 0.454603 / 3.182 | 0.140217 / 4.938 | 0.056992 / 2.413 | 0.179570 / 4.386 |
| macOS system | 3.614245 / 25.295 | 2.498502 / 87.995 | 2.221884 / 94.070 | 3.766241 / 91.986 |
| Lab workload | 0.000000 / 0.000 | 0.001975 / 0.070 | 0.000624 / 0.026 | 0.000000 / 0.000 |
| Other dependencies | 9.897470 / 69.271 | 0.030304 / 1.067 | 0.025040 / 1.060 | 0.025254 / 0.617 |
| Unresolved | 0.011114 / 0.078 | 0.001537 / 0.054 | 0.003043 / 0.129 | 0.001088 / 0.027 |

Typing's 600 input events led to 519 presents; its 599 prepaint calls therefore have a different denominator. Its 14.288096 process CPU ms/frame includes work in input handlers that the draw+present timer does not cover. `compute_injection_layers` is **9.108933 ms/frame (63.752%)**, under Kit `update_edits` at **9.736435 ms/frame (68.144%)**. The Tree-sitter C functions are charged to Other dependencies in the exclusive table; nearest-framework-caller attribution charges **10.234994 ms/frame** to Kit. Moving this work to another thread alone would not remove its process CPU cost.

Hover and drag are roughly 120 attempted presents/s; idle is about 1.9/s. Their per-attempt costs should not be interpreted as equal CPU per second. Only log was repeated for rank stability; all other scene rankings have one capture.


### typing-1 — prepaint CPU

| Component | CPU ms/attempt |
|---|---:|
| other prepaint | 0.497687 |
| phase recorder, including formatting and writes | 0.252235 |
| hit testing | 0.000587 |
| layout engine, excluding recorder | 0.001033 |

### typing-1 — top 20 inclusive, unpruned

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `UNRESOLVED HIToolbox+0x24137f` | macOS system | 13.404252 | 93.814 | — |
| `UNRESOLVED AppKit+0x37c28f` | macOS system | 13.404252 | 93.814 | — |
| `UNRESOLVED AppKit+0x394fb` | macOS system | 13.404252 | 93.814 | — |
| `UNRESOLVED AppKit+0x8b16ff` | macOS system | 13.404252 | 93.814 | — |
| `UNRESOLVED AppKit+0x8b1223` | macOS system | 13.404252 | 93.814 | — |
| `UNRESOLVED AppKit+0x2c90b` | macOS system | 13.404252 | 93.814 | — |
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 13.404252 | 93.814 | [crates/rustui-gpui-fast-macos/src/platform.rs:559](../../crates/rustui-gpui-fast-macos/src/platform.rs#L559) |
| `rustui_lab::g0::run_cli` | Lab workload | 13.404252 | 93.814 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 13.404252 | 93.814 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 13.404252 | 93.814 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 13.404252 | 93.814 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 13.404252 | 93.814 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 13.404252 | 93.814 | — |
| `start` | macOS system | 13.404252 | 93.814 | — |
| `UNRESOLVED HIToolbox+0xbe91b` | macOS system | 13.399557 | 93.781 | — |
| `UNRESOLVED HIToolbox+0xbb58f` | macOS system | 13.397412 | 93.766 | — |
| `UNRESOLVED CoreFoundation+0x15a643` | macOS system | 13.384956 | 93.679 | — |
| `<gpui::app::App>::update_window_erased` | GPUI Fast | 11.905511 | 83.325 | [crates/rustui-gpui-fast/src/app.rs:3273](../../crates/rustui-gpui-fast/src/app.rs#L3273) |
| `UNRESOLVED libdispatch.dylib+0x1b59b` | macOS system | 10.354547 | 72.470 | — |
| `UNRESOLVED libdispatch.dylib+0x10a43` | macOS system | 10.351237 | 72.447 | — |

### typing-1 — top 20 named inclusive functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 13.404252 | 93.814 | [crates/rustui-gpui-fast-macos/src/platform.rs:559](../../crates/rustui-gpui-fast-macos/src/platform.rs#L559) |
| `rustui_lab::g0::run_cli` | Lab workload | 13.404252 | 93.814 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 13.404252 | 93.814 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 13.404252 | 93.814 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 13.404252 | 93.814 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 13.404252 | 93.814 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 13.404252 | 93.814 | — |
| `start` | macOS system | 13.404252 | 93.814 | — |
| `<gpui::app::App>::update_window_erased` | GPUI Fast | 11.905511 | 83.325 | [crates/rustui-gpui-fast/src/app.rs:3273](../../crates/rustui-gpui-fast/src/app.rs#L3273) |
| `gpui_apple::dispatcher::trampoline` | GPUI Fast | 10.349066 | 72.431 | — |
| `<async_task::raw::RawTask<scheduler::executor::spawn_local_with_source_location::Checked<core::pin::Pin<alloc::boxed::Box<dyn core::future::future::Future<Output = ()>>>>, (), <scheduler::executor::LocalExecutor>::schedule::{closure#0}, scheduler::RunnableMeta>>::run` | Other dependencies | 10.349029 | 72.431 | [vendor/async-task-4.7.1/src/raw.rs:557](../../vendor/async-task-4.7.1/src/raw.rs#L557) |
| `rustui_lab::g0::playback::{closure#0}::{closure#0}::<i32>` | Lab workload | 10.316784 | 72.205 | [apps/rustui-lab/src/g0.rs:336](../../apps/rustui-lab/src/g0.rs#L336) |
| `<gpui::app::App>::spawn::<rustui_lab::g0::playback::{closure#0}, ()>::{closure#0}` | GPUI Fast | 10.316784 | 72.205 | [crates/rustui-gpui-fast/src/app.rs:2464](../../crates/rustui-gpui-fast/src/app.rs#L2464) |
| `<gpui::window::Window>::dispatch_keystroke` | GPUI Fast | 10.224156 | 71.557 | [crates/rustui-gpui-fast/src/window.rs:6109](../../crates/rustui-gpui-fast/src/window.rs#L6109) |
| `<gpui::app::App>::update_window_id::<bool, rustui_lab::g0::playback::{closure#0}::{closure#0}::{closure#11}>::{closure#0}` | GPUI Fast | 10.224156 | 71.557 | [crates/rustui-gpui-fast/src/app.rs:2368](../../crates/rustui-gpui-fast/src/app.rs#L2368) |
| `<gpui::app::App>::update_entity_erased` | GPUI Fast | 10.043545 | 70.293 | [crates/rustui-gpui-fast/src/app.rs:3335](../../crates/rustui-gpui-fast/src/app.rs#L3335) |
| `<gpui_base::input::state::InputBaseState<gpui_base::input::kind::EditorMode> as gpui::input::EntityInputHandler>::replace_text_in_range` | GPUI Kit | 9.925869 | 69.469 | — |
| `<gpui_base::input::state::InputBaseState<gpui_base::input::kind::EditorMode>>::replace_text_in_ranges` | GPUI Kit | 9.924890 | 69.463 | [crates/rustui-gpui-base/src/input/base/state.rs:4329](../../crates/rustui-gpui-base/src/input/base/state.rs#L4329) |
| `<gpui_component::highlighter::highlighter::SyntaxHighlighter>::update_edits` | GPUI Kit | 9.736435 | 68.144 | [crates/rustui-gpui-component/src/highlighter/highlighter.rs:648](../../crates/rustui-gpui-component/src/highlighter/highlighter.rs#L648) |
| `<gpui_component::highlighter::input_adapter::TreeSitterInputHighlighter as gpui_base::input::highlighting::InputHighlighter>::update` | GPUI Kit | 9.736435 | 68.144 | [crates/rustui-gpui-component/src/highlighter/input_adapter.rs:160](../../crates/rustui-gpui-component/src/highlighter/input_adapter.rs#L160) |

### typing-1 — top 20 self-time functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `ts_tree_cursor_child_iterator_next` | Other: Tree-sitter | 2.906501 | 20.342 | — |
| `ts_query_cursor__advance` | Other: Tree-sitter | 2.729118 | 19.101 | — |
| `ts_tree_cursor_current_status` | Other: Tree-sitter | 2.021164 | 14.146 | — |
| `mach_msg2_trap` | macOS system | 1.323753 | 9.265 | — |
| `write` | macOS system | 0.897218 | 6.279 | — |
| `__workq_kernreturn` | macOS system | 0.878044 | 6.145 | — |
| `ts_tree_cursor_goto_sibling_internal` | Other: Tree-sitter | 0.728000 | 5.095 | — |
| `ts_tree_cursor_goto_first_child_internal` | Other: Tree-sitter | 0.564620 | 3.952 | — |
| `ts_query__pattern_map_search` | Other: Tree-sitter | 0.253008 | 1.771 | — |
| `ts_parser_parse` | Other: Tree-sitter | 0.203403 | 1.424 | — |
| `_platform_memmove` | macOS system | 0.174455 | 1.221 | — |
| `ts_subtree_summarize_children` | Other: Tree-sitter | 0.125952 | 0.882 | — |
| `ts_stack_push` | Other: Tree-sitter | 0.049482 | 0.346 | — |
| `<gpui::scene::Scene>::insert_primitive::<gpui::scene::MonochromeSprite>` | GPUI Fast | 0.043013 | 0.301 | [crates/rustui-gpui-fast/src/scene.rs:93](../../crates/rustui-gpui-fast/src/scene.rs#L93) |
| `gpui::text_system::line::paint_line` | GPUI Fast | 0.037150 | 0.260 | [crates/rustui-gpui-fast/src/text_system/line.rs:559](../../crates/rustui-gpui-fast/src/text_system/line.rs#L559) |
| `stack__iter` | Other dependencies | 0.035333 | 0.247 | — |
| `ts_tree_cursor_goto_parent` | Other: Tree-sitter | 0.027333 | 0.191 | — |
| `UNRESOLVED libsystem_malloc.dylib+0x33a48` | std/allocator | 0.023054 | 0.161 | — |
| `_platform_memcmp` | macOS system | 0.022921 | 0.160 | — |
| `kevent_qos` | macOS system | 0.019834 | 0.139 | — |

### hover-1 — prepaint CPU

| Component | CPU ms/attempt |
|---|---:|
| phase recorder, including formatting and writes | 0.952140 |
| other prepaint | 0.436646 |
| layout engine, excluding recorder | 0.004155 |
| hit testing | 0.000614 |

### hover-1 — top 20 inclusive, unpruned

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `UNRESOLVED HIToolbox+0x24137f` | macOS system | 2.441889 | 86.001 | — |
| `UNRESOLVED AppKit+0x37c28f` | macOS system | 2.441889 | 86.001 | — |
| `UNRESOLVED AppKit+0x394fb` | macOS system | 2.441889 | 86.001 | — |
| `UNRESOLVED AppKit+0x8b16ff` | macOS system | 2.441889 | 86.001 | — |
| `UNRESOLVED AppKit+0x8b1223` | macOS system | 2.441889 | 86.001 | — |
| `UNRESOLVED AppKit+0x2c90b` | macOS system | 2.441889 | 86.001 | — |
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 2.441889 | 86.001 | [crates/rustui-gpui-fast-macos/src/platform.rs:563](../../crates/rustui-gpui-fast-macos/src/platform.rs#L563) |
| `rustui_lab::g0::run_cli` | Lab workload | 2.441889 | 86.001 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 2.441889 | 86.001 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 2.441889 | 86.001 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 2.441889 | 86.001 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 2.441889 | 86.001 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 2.441889 | 86.001 | — |
| `start` | macOS system | 2.441889 | 86.001 | — |
| `UNRESOLVED HIToolbox+0xbe91b` | macOS system | 2.441541 | 85.989 | — |
| `UNRESOLVED HIToolbox+0xbb58f` | macOS system | 2.440370 | 85.948 | — |
| `UNRESOLVED CoreFoundation+0x15a643` | macOS system | 2.438790 | 85.892 | — |
| `<gpui::app::App>::update_window_erased` | GPUI Fast | 1.967077 | 69.279 | [crates/rustui-gpui-fast/src/app.rs:3268](../../crates/rustui-gpui-fast/src/app.rs#L3268) |
| `UNRESOLVED QuartzCore+0x62b4f` | macOS system | 1.966617 | 69.263 | — |
| `<gpui_apple::display_link::MetalDisplayLinkDelegate>::needs_update` | GPUI Fast | 1.963522 | 69.154 | [crates/rustui-gpui-fast-apple/src/display_link.rs:28](../../crates/rustui-gpui-fast-apple/src/display_link.rs#L28) |

### hover-1 — top 20 named inclusive functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 2.441889 | 86.001 | [crates/rustui-gpui-fast-macos/src/platform.rs:563](../../crates/rustui-gpui-fast-macos/src/platform.rs#L563) |
| `rustui_lab::g0::run_cli` | Lab workload | 2.441889 | 86.001 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 2.441889 | 86.001 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 2.441889 | 86.001 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 2.441889 | 86.001 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 2.441889 | 86.001 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 2.441889 | 86.001 | — |
| `start` | macOS system | 2.441889 | 86.001 | — |
| `<gpui::app::App>::update_window_erased` | GPUI Fast | 1.967077 | 69.279 | [crates/rustui-gpui-fast/src/app.rs:3268](../../crates/rustui-gpui-fast/src/app.rs#L3268) |
| `<gpui_apple::display_link::MetalDisplayLinkDelegate>::needs_update` | GPUI Fast | 1.963522 | 69.154 | [crates/rustui-gpui-fast-apple/src/display_link.rs:28](../../crates/rustui-gpui-fast-apple/src/display_link.rs#L28) |
| `<gpui_macos::window::MacWindowState>::start_display_link::{closure#0}` | GPUI Fast | 1.963516 | 69.153 | [crates/rustui-gpui-fast-macos/src/window.rs:856](../../crates/rustui-gpui-fast-macos/src/window.rs#L856) |
| `<gpui::window::Window>::new::{closure#10}` | GPUI Fast | 1.963445 | 69.151 | [crates/rustui-gpui-fast/src/window.rs:1957](../../crates/rustui-gpui-fast/src/window.rs#L1957) |
| `<gpui::window::Window>::new::{closure#10}::{closure#6}` | GPUI Fast | 1.959650 | 69.017 | [crates/rustui-gpui-fast/src/window.rs:2100](../../crates/rustui-gpui-fast/src/window.rs#L2100) |
| `<gpui::app::App>::update_window_id::<(), <gpui::window::Window>::new::{closure#10}::{closure#6}::{closure#0}>::{closure#0}` | GPUI Fast | 1.958470 | 68.976 | [crates/rustui-gpui-fast/src/app.rs:2366](../../crates/rustui-gpui-fast/src/app.rs#L2366) |
| `<gpui::window::Window>::draw` | GPUI Fast | 1.840703 | 64.828 | [crates/rustui-gpui-fast/src/window.rs:3904](../../crates/rustui-gpui-fast/src/window.rs#L3904) |
| `<gpui::window::Window>::draw_roots` | GPUI Fast | 1.758867 | 61.946 | [crates/rustui-gpui-fast/src/window.rs:4160](../../crates/rustui-gpui-fast/src/window.rs#L4160) |
| `core::fmt::write` | std/allocator | 1.556944 | 54.834 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/fmt/mod.rs:1631` |
| `gpui::window::f0_frame_record` | GPUI Fast | 1.556057 | 54.803 | — |
| `std::io::stdio::_eprint` | std/allocator | 1.554532 | 54.749 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/io/stdio.rs:1284` |
| `<&std::io::stdio::Stderr as std::io::Write>::write_fmt` | std/allocator | 1.554407 | 54.745 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/io/stdio.rs:1066` |

### hover-1 — top 20 self-time functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `write` | macOS system | 1.540372 | 54.251 | — |
| `mach_msg2_trap` | macOS system | 0.460008 | 16.201 | — |
| `__workq_kernreturn` | macOS system | 0.387489 | 13.647 | — |
| `_platform_memmove` | macOS system | 0.037007 | 1.303 | — |
| `gpui::text_system::line::paint_line` | GPUI Fast | 0.011208 | 0.395 | [crates/rustui-gpui-fast/src/text_system/line.rs:577](../../crates/rustui-gpui-fast/src/text_system/line.rs#L577) |
| `<hashbrown::map::HashMap<(gpui::element::GlobalElementId, core::any::TypeId), gpui::window::ElementStateBox, rustc_hash::FxBuildHasher>>::remove_entry::<(gpui::element::GlobalElementId, core::any::TypeId)>` | Other dependencies | 0.010294 | 0.363 | `/rust/deps/hashbrown-0.17.1/src/map.rs:1992` |
| `kevent_id` | macOS system | 0.009218 | 0.325 | — |
| `<gpui::scene::Scene>::insert_primitive::<gpui::scene::MonochromeSprite>` | GPUI Fast | 0.008012 | 0.282 | [crates/rustui-gpui-fast/src/scene.rs:94](../../crates/rustui-gpui-fast/src/scene.rs#L94) |
| `<gpui::elements::div::Interactivity>::request_layout::<<gpui::elements::div::Div as gpui::element::Element>::request_layout::{closure#1}::{closure#0}>::{closure#0}` | GPUI Fast | 0.006999 | 0.247 | [crates/rustui-gpui-fast/src/elements/div.rs:2339](../../crates/rustui-gpui-fast/src/elements/div.rs#L2339) |
| `<gpui::style::Style as refineable::Refineable>::refine` | GPUI Fast | 0.006834 | 0.241 | [crates/rustui-gpui-fast/src/style.rs:179](../../crates/rustui-gpui-fast/src/style.rs#L179) |
| `<gpui::elements::div::Interactivity>::compute_style_internal` | GPUI Fast | 0.006414 | 0.226 | [crates/rustui-gpui-fast/src/elements/div.rs:3557](../../crates/rustui-gpui-fast/src/elements/div.rs#L3557) |
| `_platform_memcmp` | macOS system | 0.005299 | 0.187 | — |
| `gpui::fast::layout::layout_fingerprint` | GPUI Fast | 0.004602 | 0.162 | [crates/rustui-gpui-fast/src/fast/layout.rs:1360](../../crates/rustui-gpui-fast/src/fast/layout.rs#L1360) |
| `<gpui_component::list::list_item::ListItem as gpui::element::RenderOnce>::render` | GPUI Kit | 0.004419 | 0.156 | [crates/rustui-gpui-component/src/list/list_item.rs:188](../../crates/rustui-gpui-component/src/list/list_item.rs#L188) |
| `gpui_macos::platform::build_classes::_::__ctor_private` | GPUI Fast | 0.004410 | 0.155 | — |
| `<smol_str::SmolStr as core::cmp::PartialEq>::eq` | Other dependencies | 0.004375 | 0.154 | [vendor/smol_str-0.3.6/src/lib.rs:145](../../vendor/smol_str-0.3.6/src/lib.rs#L145) |
| `UNRESOLVED libsystem_malloc.dylib+0x3389c` | std/allocator | 0.003967 | 0.140 | — |
| `<gpui::window::Window>::draw` | GPUI Fast | 0.003879 | 0.137 | [crates/rustui-gpui-fast/src/window.rs:3904](../../crates/rustui-gpui-fast/src/window.rs#L3904) |
| `<gpui::elements::div::Interactivity>::paint::<<gpui::elements::div::Div as gpui::element::Element>::paint::{closure#1}::{closure#0}>::{closure#1}` | GPUI Fast | 0.003845 | 0.135 | [crates/rustui-gpui-fast/src/elements/div.rs:2627](../../crates/rustui-gpui-fast/src/elements/div.rs#L2627) |
| `<gpui::elements::div::Div as gpui::element::Element>::prepaint` | GPUI Fast | 0.003648 | 0.128 | [crates/rustui-gpui-fast/src/elements/div.rs:2046](../../crates/rustui-gpui-fast/src/elements/div.rs#L2046) |

### drag-1 — prepaint CPU

| Component | CPU ms/attempt |
|---|---:|
| phase recorder, including formatting and writes | 0.085917 |
| other prepaint | 0.101251 |
| hit testing | 0.000009 |

### drag-1 — top 20 inclusive, unpruned

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `UNRESOLVED HIToolbox+0x24137f` | macOS system | 1.652244 | 69.952 | — |
| `UNRESOLVED AppKit+0x37c28f` | macOS system | 1.652244 | 69.952 | — |
| `UNRESOLVED AppKit+0x394fb` | macOS system | 1.652244 | 69.952 | — |
| `UNRESOLVED AppKit+0x8b16ff` | macOS system | 1.652244 | 69.952 | — |
| `UNRESOLVED AppKit+0x8b1223` | macOS system | 1.652244 | 69.952 | — |
| `UNRESOLVED AppKit+0x2c90b` | macOS system | 1.652244 | 69.952 | — |
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 1.652244 | 69.952 | [crates/rustui-gpui-fast-macos/src/platform.rs:563](../../crates/rustui-gpui-fast-macos/src/platform.rs#L563) |
| `rustui_lab::g0::run_cli` | Lab workload | 1.652244 | 69.952 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 1.652244 | 69.952 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 1.652244 | 69.952 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 1.652244 | 69.952 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 1.652244 | 69.952 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 1.652244 | 69.952 | — |
| `start` | macOS system | 1.652244 | 69.952 | — |
| `UNRESOLVED HIToolbox+0xbe91b` | macOS system | 1.651481 | 69.920 | — |
| `UNRESOLVED HIToolbox+0xbb58f` | macOS system | 1.650434 | 69.876 | — |
| `UNRESOLVED CoreFoundation+0x15a643` | macOS system | 1.648407 | 69.790 | — |
| `<gpui::app::App>::update_window_erased` | GPUI Fast | 0.847358 | 35.875 | [crates/rustui-gpui-fast/src/app.rs:3268](../../crates/rustui-gpui-fast/src/app.rs#L3268) |
| `UNRESOLVED QuartzCore+0x62b4f` | macOS system | 0.845493 | 35.796 | — |
| `UNRESOLVED QuartzCore+0xea1db` | macOS system | 0.842481 | 35.669 | — |

### drag-1 — top 20 named inclusive functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 1.652244 | 69.952 | [crates/rustui-gpui-fast-macos/src/platform.rs:563](../../crates/rustui-gpui-fast-macos/src/platform.rs#L563) |
| `rustui_lab::g0::run_cli` | Lab workload | 1.652244 | 69.952 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 1.652244 | 69.952 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 1.652244 | 69.952 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 1.652244 | 69.952 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 1.652244 | 69.952 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 1.652244 | 69.952 | — |
| `start` | macOS system | 1.652244 | 69.952 | — |
| `<gpui::app::App>::update_window_erased` | GPUI Fast | 0.847358 | 35.875 | [crates/rustui-gpui-fast/src/app.rs:3268](../../crates/rustui-gpui-fast/src/app.rs#L3268) |
| `<gpui_apple::display_link::MetalDisplayLinkDelegate>::needs_update` | GPUI Fast | 0.842308 | 35.662 | [vendor/objc2-0.6.5/src/macros/define_class.rs:1329](../../vendor/objc2-0.6.5/src/macros/define_class.rs#L1329) |
| `<gpui_macos::window::MacWindowState>::start_display_link::{closure#0}` | GPUI Fast | 0.841968 | 35.647 | [crates/rustui-gpui-fast-macos/src/window.rs:856](../../crates/rustui-gpui-fast-macos/src/window.rs#L856) |
| `<gpui::window::Window>::new::{closure#10}` | GPUI Fast | 0.841688 | 35.635 | [crates/rustui-gpui-fast/src/window.rs:1999](../../crates/rustui-gpui-fast/src/window.rs#L1999) |
| `<gpui::window::Window>::new::{closure#10}::{closure#6}` | GPUI Fast | 0.834488 | 35.330 | [crates/rustui-gpui-fast/src/window.rs:2100](../../crates/rustui-gpui-fast/src/window.rs#L2100) |
| `<gpui::app::App>::update_window_id::<(), <gpui::window::Window>::new::{closure#10}::{closure#6}::{closure#0}>::{closure#0}` | GPUI Fast | 0.833899 | 35.305 | [crates/rustui-gpui-fast/src/app.rs:2368](../../crates/rustui-gpui-fast/src/app.rs#L2368) |
| `mach_msg2_trap` | macOS system | 0.786712 | 33.308 | — |
| `mach_msg2_internal` | macOS system | 0.786712 | 33.308 | — |
| `mach_msg_overwrite` | macOS system | 0.785523 | 33.257 | — |
| `mach_msg` | macOS system | 0.785523 | 33.257 | — |
| `_pthread_wqthread` | macOS system | 0.707382 | 29.949 | — |
| `<gpui::window::Window>::draw` | GPUI Fast | 0.696453 | 29.486 | [crates/rustui-gpui-fast/src/window.rs:3902](../../crates/rustui-gpui-fast/src/window.rs#L3902) |

### drag-1 — top 20 self-time functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `mach_msg2_trap` | macOS system | 0.786712 | 33.308 | — |
| `__workq_kernreturn` | macOS system | 0.687916 | 29.125 | — |
| `write` | macOS system | 0.637850 | 27.005 | — |
| `kevent_id` | macOS system | 0.012361 | 0.523 | — |
| `gpui::text_system::line::paint_line` | GPUI Fast | 0.005139 | 0.218 | [crates/rustui-gpui-fast/src/text_system/line.rs:559](../../crates/rustui-gpui-fast/src/text_system/line.rs#L559) |
| `_platform_memmove` | macOS system | 0.004351 | 0.184 | — |
| `taffy::compute::flexbox::compute_preliminary::<taffy::tree::taffy_tree::TaffyView<gpui::taffy::NodeContext, <gpui::taffy::TaffyLayoutEngine>::compute_layout::{closure#1}>>` | Other dependencies | 0.003702 | 0.157 | [vendor/taffy-0.15.0/src/compute/flexbox.rs:329](../../vendor/taffy-0.15.0/src/compute/flexbox.rs#L329) |
| `<gpui::window::Window>::draw` | GPUI Fast | 0.003481 | 0.147 | [crates/rustui-gpui-fast/src/window.rs:3902](../../crates/rustui-gpui-fast/src/window.rs#L3902) |
| `taffy::compute::block::compute_inner::<taffy::tree::taffy_tree::TaffyView<gpui::taffy::NodeContext, <gpui::taffy::TaffyLayoutEngine>::compute_layout::{closure#1}>>` | Other dependencies | 0.003342 | 0.142 | [vendor/taffy-0.15.0/src/compute/block.rs:493](../../vendor/taffy-0.15.0/src/compute/block.rs#L493) |
| `UNRESOLVED libsystem_malloc.dylib+0x33a48` | std/allocator | 0.003119 | 0.132 | — |
| `<gpui::window::Window>::draw_roots` | GPUI Fast | 0.002687 | 0.114 | [crates/rustui-gpui-fast/src/window.rs:4161](../../crates/rustui-gpui-fast/src/window.rs#L4161) |
| `__kdebug_trace64` | macOS system | 0.002525 | 0.107 | — |
| `<gpui::scene::BatchIterator as core::iter::traits::iterator::Iterator>::next` | GPUI Fast | 0.002381 | 0.101 | [crates/rustui-gpui-fast/src/scene.rs:295](../../crates/rustui-gpui-fast/src/scene.rs#L295) |
| `start_wqthread` | macOS system | 0.002328 | 0.099 | — |
| `taffy::compute::oof::perform_oof_layout::<taffy::tree::taffy_tree::TaffyView<gpui::taffy::NodeContext, <gpui::taffy::TaffyLayoutEngine>::compute_layout::{closure#1}>>` | Other dependencies | 0.002275 | 0.096 | [vendor/taffy-0.15.0/src/compute/oof.rs:736](../../vendor/taffy-0.15.0/src/compute/oof.rs#L736) |
| `UNRESOLVED libobjcMsgSend.dylib+0x844` | macOS system | 0.002022 | 0.086 | — |
| `<gpui_apple::metal_atlas::MetalAtlas as gpui::platform::PlatformAtlas>::get_or_insert_with` | GPUI Fast | 0.001975 | 0.084 | [crates/rustui-gpui-fast-apple/src/metal_atlas.rs:56](../../crates/rustui-gpui-fast-apple/src/metal_atlas.rs#L56) |
| `kevent_qos` | macOS system | 0.001968 | 0.083 | — |
| `gpui::fast::retained::paint_entity_view` | GPUI Fast | 0.001889 | 0.080 | [crates/rustui-gpui-fast/src/fast/retained.rs:1938](../../crates/rustui-gpui-fast/src/fast/retained.rs#L1938) |
| `<core::iter::adapters::map::Map<core::iter::adapters::filter::Filter<core::iter::adapters::filter::Filter<core::iter::adapters::map::Map<core::iter::adapters::enumerate::Enumerate<taffy::tree::taffy_tree::TaffyTreeChildIter>, taffy::compute::flexbox::generate_anonymous_flex_items<taffy::tree::taffy_tree::TaffyView<gpui::taffy::NodeContext, <gpui::taffy::TaffyLayoutEngine>::compute_layout::{closure#1}>>::{closure#0}>, taffy::compute::flexbox::generate_anonymous_flex_items<taffy::tree::taffy_tree::TaffyView<gpui::taffy::NodeContext, <gpui::taffy::TaffyLayoutEngine>::compute_layout::{closure#1}>>::{closure#1}>, taffy::compute::flexbox::generate_anonymous_flex_items<taffy::tree::taffy_tree::TaffyView<gpui::taffy::NodeContext, <gpui::taffy::TaffyLayoutEngine>::compute_layout::{closure#1}>>::{closure#2}>, taffy::compute::flexbox::generate_anonymous_flex_items<taffy::tree::taffy_tree::TaffyView<gpui::taffy::NodeContext, <gpui::taffy::TaffyLayoutEngine>::compute_layout::{closure#1}>>::{closure#3}> as core::iter::traits::iterator::Iterator>::next` | std/allocator | 0.001868 | 0.079 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/iter/adapters/map.rs:107` |

### idle-1 — prepaint CPU

| Component | CPU ms/attempt |
|---|---:|
| phase recorder, including formatting and writes | 0.126861 |
| other prepaint | 0.198745 |

### idle-1 — top 20 inclusive, unpruned

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `UNRESOLVED CoreFoundation+0x15a643` | macOS system | 3.088884 | 75.442 | — |
| `UNRESOLVED HIToolbox+0xbb58f` | macOS system | 3.088884 | 75.442 | — |
| `UNRESOLVED HIToolbox+0xbe91b` | macOS system | 3.088884 | 75.442 | — |
| `UNRESOLVED HIToolbox+0x24137f` | macOS system | 3.088884 | 75.442 | — |
| `UNRESOLVED AppKit+0x37c28f` | macOS system | 3.088884 | 75.442 | — |
| `UNRESOLVED AppKit+0x394fb` | macOS system | 3.088884 | 75.442 | — |
| `UNRESOLVED AppKit+0x8b16ff` | macOS system | 3.088884 | 75.442 | — |
| `UNRESOLVED AppKit+0x8b1223` | macOS system | 3.088884 | 75.442 | — |
| `UNRESOLVED AppKit+0x2c90b` | macOS system | 3.088884 | 75.442 | — |
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 3.088884 | 75.442 | [crates/rustui-gpui-fast-macos/src/platform.rs:563](../../crates/rustui-gpui-fast-macos/src/platform.rs#L563) |
| `rustui_lab::g0::run_cli` | Lab workload | 3.088884 | 75.442 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 3.088884 | 75.442 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 3.088884 | 75.442 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 3.088884 | 75.442 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 3.088884 | 75.442 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 3.088884 | 75.442 | — |
| `start` | macOS system | 3.088884 | 75.442 | — |
| `mach_msg2_trap` | macOS system | 1.680682 | 41.049 | — |
| `mach_msg2_internal` | macOS system | 1.680682 | 41.049 | — |
| `mach_msg_overwrite` | macOS system | 1.680682 | 41.049 | — |

### idle-1 — top 20 named inclusive functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` | GPUI Fast | 3.088884 | 75.442 | [crates/rustui-gpui-fast-macos/src/platform.rs:563](../../crates/rustui-gpui-fast-macos/src/platform.rs#L563) |
| `rustui_lab::g0::run_cli` | Lab workload | 3.088884 | 75.442 | [apps/rustui-lab/src/g0.rs:405](../../apps/rustui-lab/src/g0.rs#L405) |
| `rustui_lab::main` | Lab workload | 3.088884 | 75.442 | [apps/rustui-lab/src/main.rs:6](../../apps/rustui-lab/src/main.rs#L6) |
| `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` | std/allocator | 3.088884 | 75.442 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/sys/backtrace.rs:166` |
| `std::rt::lang_start::<()>::{closure#0}` | std/allocator | 3.088884 | 75.442 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:206` |
| `std::rt::lang_start_internal` | std/allocator | 3.088884 | 75.442 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/std/src/rt.rs:171` |
| `main` | Lab workload | 3.088884 | 75.442 | — |
| `start` | macOS system | 3.088884 | 75.442 | — |
| `mach_msg2_trap` | macOS system | 1.680682 | 41.049 | — |
| `mach_msg2_internal` | macOS system | 1.680682 | 41.049 | — |
| `mach_msg_overwrite` | macOS system | 1.680682 | 41.049 | — |
| `mach_msg` | macOS system | 1.680682 | 41.049 | — |
| `<gpui::app::App>::update_window_id::<(), <gpui::window::Window>::new::{closure#10}::{closure#6}::{closure#0}>::{closure#0}` | GPUI Fast | 1.391386 | 33.983 | [crates/rustui-gpui-fast/src/app.rs:2368](../../crates/rustui-gpui-fast/src/app.rs#L2368) |
| `<gpui::app::App>::update_window_erased` | GPUI Fast | 1.391386 | 33.983 | [crates/rustui-gpui-fast/src/app.rs:3273](../../crates/rustui-gpui-fast/src/app.rs#L3273) |
| `<gpui::window::Window>::new::{closure#10}::{closure#6}` | GPUI Fast | 1.391386 | 33.983 | [crates/rustui-gpui-fast/src/window.rs:2102](../../crates/rustui-gpui-fast/src/window.rs#L2102) |
| `<gpui::window::Window>::new::{closure#10}` | GPUI Fast | 1.391386 | 33.983 | [crates/rustui-gpui-fast/src/window.rs:2100](../../crates/rustui-gpui-fast/src/window.rs#L2100) |
| `<gpui_macos::window::MacWindowState>::start_display_link::{closure#0}` | GPUI Fast | 1.391386 | 33.983 | [crates/rustui-gpui-fast-macos/src/window.rs:858](../../crates/rustui-gpui-fast-macos/src/window.rs#L858) |
| `<gpui_apple::display_link::MetalDisplayLinkDelegate>::needs_update` | GPUI Fast | 1.391386 | 33.983 | [crates/rustui-gpui-fast-apple/src/display_link.rs:29](../../crates/rustui-gpui-fast-apple/src/display_link.rs#L29) |
| `<gpui::window::Window>::draw` | GPUI Fast | 1.277412 | 31.199 | [crates/rustui-gpui-fast/src/window.rs:3944](../../crates/rustui-gpui-fast/src/window.rs#L3944) |
| `gpui::window::f0_frame_record` | GPUI Fast | 1.016623 | 24.830 | — |

### idle-1 — top 20 self-time functions

| Function / unresolved frame | Crate group | CPU ms/frame | % of frame CPU | Sampled source |
|---|---|---:|---:|---|
| `mach_msg2_trap` | macOS system | 1.680682 | 41.049 | — |
| `__workq_kernreturn` | macOS system | 0.996006 | 24.326 | — |
| `write` | macOS system | 0.987886 | 24.128 | — |
| `gpui_apple::fast::layers::raster::rasterize_tiles` | GPUI Fast | 0.018254 | 0.446 | [crates/rustui-gpui-fast-apple/src/fast/layers/raster.rs:52](../../crates/rustui-gpui-fast-apple/src/fast/layers/raster.rs#L52) |
| `gpui::fast::scene::sort_sprites::<gpui::scene::SubpixelSprite, gpui::fast::scene::sort_in_drawing_order::{closure#5}>` | GPUI Fast | 0.014947 | 0.365 | — |
| `<gpui_apple::metal_renderer::MetalRenderer>::draw` | GPUI Fast | 0.012289 | 0.300 | [crates/rustui-gpui-fast-apple/src/metal_renderer.rs:640](../../crates/rustui-gpui-fast-apple/src/metal_renderer.rs#L640) |
| `UNRESOLVED libsystem_malloc.dylib+0x3bd88` | std/allocator | 0.011939 | 0.292 | — |
| `UNRESOLVED Foundation+0x135f8` | macOS system | 0.011675 | 0.285 | — |
| `UNRESOLVED AGXMetalG13X+0x25939c` | macOS system | 0.011570 | 0.283 | — |
| `<gpui::scene::Scene>::clear` | GPUI Fast | 0.011570 | 0.283 | [crates/rustui-gpui-fast/src/scene.rs:64](../../crates/rustui-gpui-fast/src/scene.rs#L64) |
| `rustui::theme::registry::active_theme` | RustUI-owned | 0.011158 | 0.273 | [crates/rustui/src/theme/registry.rs:602](../../crates/rustui/src/theme/registry.rs#L602) |
| `UNRESOLVED Metal+0x18170` | macOS system | 0.010491 | 0.256 | — |
| `<gpui_component::input::input::Input as gpui::element::RenderOnce>::render` | GPUI Kit | 0.010412 | 0.254 | [crates/rustui-gpui-component/src/input/input.rs:583](../../crates/rustui-gpui-component/src/input/input.rs#L583) |
| `UNRESOLVED libsystem_malloc.dylib+0x3f360` | std/allocator | 0.010395 | 0.254 | — |
| `<gpui::fast::dependencies::RenderDependencies>::union` | GPUI Fast | 0.010219 | 0.250 | [crates/rustui-gpui-fast/src/fast/dependencies.rs:1302](../../crates/rustui-gpui-fast/src/fast/dependencies.rs#L1302) |
| `core::fmt::write` | std/allocator | 0.010053 | 0.246 | `/rustc/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/fmt/mod.rs:1687` |
| `kevent_id` | macOS system | 0.010009 | 0.244 | — |
| `UNRESOLVED AGXMetalG13X+0x3bde24` | macOS system | 0.009877 | 0.241 | — |
| `<gpui_apple::metal_renderer::InstanceBufferWriter>::finish` | GPUI Fast | 0.009404 | 0.230 | [crates/rustui-gpui-fast-apple/src/metal_renderer.rs:1891](../../crates/rustui-gpui-fast-apple/src/metal_renderer.rs#L1891) |
| `UNRESOLVED libsystem_malloc.dylib+0x3392c` | std/allocator | 0.009219 | 0.225 | — |

## Source audit of the top RustUI-owned functions

These are **source operations per invocation/branch**, not measured allocation or clone counters. Sampling does not provide invocation counts. Retention means a lowering invocation is not necessarily executed on every attempted present; multiplying a source count by frame count would invent a measurement. No explicit `Arc::clone` call appears in these inspected RustUI hot bodies; `Rc`, `Entity` and `SharedString` clones are distinguished below rather than treating every clone as an Arc operation.

| Function and sampled inclusive cost | Rc/shared clones and RefCell operations | Maps, formatting, collect and allocation sites |
|---|---|---|
| [ComponentHost::render:387](../../crates/rustui/src/runtime/component_host.rs#L387): typing 0.098428, hover 0.034516, idle 0.054184 ms/attempt | Two explicit `Rc::new` closure/owner wrappers per host render; cloned button theme and localization when present. | Calls retained-entity bookkeeping. [get:206](../../crates/rustui/src/runtime/component_host.rs#L206) creates a scoped key, inserts a cloned key in `seen`, and looks up `entries`: one HashSet insertion plus one HashMap lookup per retained get. Two scope-Vec clones can allocate when scope is nonempty; empty Vec clones do not allocate. [finish_frame:129](../../crates/rustui/src/runtime/component_host.rs#L129) scans retained entries with HashSet membership and preserved-prefix checks. No direct `format!`/`collect()` in the host render body. Dynamic gets/scopes per frame were not counted. |
| [CodeEditorView::into_element:208](../../crates/rustui/src/components/advanced/code_editor.rs#L208): typing 0.089640 ms/attempt | [Binding::snapshot:70](../../crates/rustui-core/src/binding.rs#L70) clones an `Rc<String>`, not its bytes. Steady lowering explicitly clones previous-owner-value Rc, native-change Rc and writer Rc, plus the editor Entity and placeholder/shared labels. [replace_event_writer:26](../../crates/rustui/src/components/advanced/code_editor.rs#L26) performs one `borrow_mut`; the change callback takes a short immutable borrow and clones its current writer. Creation-only Rc/RefCell construction is outside the steady path. | No active intermediate `collect()` in this G0 path; the `RUSTUI_F0_PIXELS` branch is disabled. Conditional owner reconciliation calls `native.value()`. On a native Change event, [line 287](../../crates/rustui/src/components/advanced/code_editor.rs#L287) calls `value().to_string()`. [Kit value:1671](../../crates/rustui-gpui-base/src/input/base/state.rs#L1671) first materializes the Rope into a SharedString; `.to_string()` then creates the owned writer value. This is an avoidable intermediate materialization candidate, not a measured allocation count or a separately resolved CPU cost. |
| [ListView::into_element:370](../../crates/rustui/src/components/data/list.rs#L370): hover 0.016932 ms/attempt | Clones owner-selection, requested-selection and updater Rc handles, plus callback handles. Replaces a RefCell-held updater once per lowering. Retained lookup has the map operations above. | [resolve_list_items:72](../../crates/rustui/src/components/data/list.rs#L72) always collects a new Vec and clones each plain String; G0 has 5,000 labels per lowering invocation. `format!` for localized suffixes is inactive in G0. [ListView::new:246](../../crates/rustui/src/components/data/list.rs#L246) also collects the input iterator. Extra item clones occur only on `data_changed`. These are per-lowering source counts, not 5,000 allocations per attempted present. |
| [StringListDelegate::render_item:182](../../crates/rustui/src/components/data/list.rs#L182): hover 0.009508 ms/attempt | Clones the source String once, converts it to SharedString, then clones that shared label twice for accessibility. | Builds one ListItem/Div subtree per invoked row; no intermediate Vec collect or `format!` in this body. String and style construction can allocate; actual allocations/visible-row invocations were not counted. |
| [TextFieldView::into_element:478](../../crates/rustui/src/components/input/text_field.rs#L478): idle 0.020904 ms/attempt | Clones retained Entity/writer handles; writer replacement uses RefCell. Some focus Rc borrows occur only if an external focus target is configured. | `format!("text-field-focus:{identity}")` on each lowering; two `Binding::value()` calls clone the owned value, plus validation and previous-value copies. Placeholder `.to_string()` and `state.value().to_string()` materialize strings on the ordinary path; IME/owner-sync branches can add copies. Constructor `Vec::new` or `Rc::new` sites are not evidence of steady allocations per frame. |
| [active_theme:602](../../crates/rustui/src/theme/registry.rs#L602): idle 0.011158 ms/attempt | No explicit clone, formatting or collect in this function. | One `App::try_global::<ActiveTheme>()` lookup; downstream callers decide whether to clone the returned theme. Small idle sample counts make this rank coarse. |

**Allocations per frame: not measured.** Xcode's Allocations template was available, but its launch never progressed to a timed Lab window, remained idle past the requested 70-second limit, and did not yield window markers. The session was terminated after more than two minutes. No allocation count is inferred from sampled allocator CPU, `Vec::new`, or source-level clone sites. No runtime instrumentation was added.

## The biggest measured bottleneck and the smallest next fix

**One primary bottleneck: GPUI Kit's full-tree Rust injection discovery on each local edit**, [highlighter.rs:782–783](../../crates/rustui-gpui-component/src/highlighter/highlighter.rs#L782), called unconditionally after a successful incremental parse at [line 685](../../crates/rustui-gpui-component/src/highlighter/highlighter.rs#L685). It costs **9.108933 inclusive CPU ms/attempt, 63.752% of typing process CPU**, the largest actionable subtree measured across these scenes. This is 4.727536 CPU seconds over the capture (519 attempts), or about 7.879 ms per logged input event (600); the latter is input normalization, not a measured function-call duration. The pinned [Rust injection query](../../crates/rustui-gpui-component/src/highlighter/languages/rust/injections.scm) contains only rooted `macro_invocation`/`macro_rule` token-tree patterns with fixed Rust language properties. The G0 workload changes the leading ordinary comment.

**Smallest next fix to test:** add a conservative Rust-specific negative-result fast path at `update_edits` before line 685. Cache whether the prior pinned Rust injection query produced **zero matches**, then reuse its empty result only when the old/new edited trees report no structural changed ranges. An empty parsed-layer list alone is insufficient: query matches can be filtered, unsupported or capped. Any prior query match keeps the existing full scan, even if no layer was produced. Restrict the fast path to the exact pinned, text-independent Rust query; query/grammar changes, structural changes, ambiguous edit batches and other languages keep the current full-scan fallback. This deliberately does not optimize files containing macro candidates. Do not disable Rust macro injection highlighting globally, and do not apply this argument to Markdown's text-dependent fence language predicates.

This is a **proposal, not implemented or proven by P0**. Validate Tree-sitter's changed-range semantics for the actual edits before relying on the fast path. Oracle tests must compare reused versus freshly rebuilt highlighting for comment/identifier edits, macro creation/removal, nested token-tree edits, delimiter/newline changes, undo and grammar/query changes. Filtered or over-budget macro candidates must never be mistaken for a zero-match query result. Any failed precondition falls back to the existing scan.

**Expected saving:** up to the measured 9.109 ms/attempt is the observed upper bound for eliminating this subtree on G0's non-injection edits. An exploratory target is **5–8 ms process CPU/attempt**, not a claimed result or guaranteed saving. Actual parsing, visible style queries and normal rendering remain. **Ownership: GPUI Kit, requiring an owner patch decision**; this is not an allowed RustUI-owned Stage P runtime edit. P0 does not authorize the patch or start P1.

Measure an approved candidate with one release build and an off-by-default test switch: five matched off/on pairs per scene, alternating off→on, on→off, off→on, on→off, off→on, 60 seconds after normal warmup, damage off, no compiler/profiler/other load during timed pairs. Keep logging flags identical across arms. Report all five changes, median/min/max, and preserve the existing phase-CPU verdict: PASS iff all five ≤3%, FAIL iff all five >3%, otherwise INCONCLUSIVE. Also collect measured-window process user+system CPU/attempt as a diagnostic—this input-handler bottleneck largely lies outside draw+present timing, so that phase metric alone cannot demonstrate its removal. An external process-CPU observer or owner-approved existing window instrumentation is needed for that future measurement; none is added by P0. Reprofile separately after timing to verify that injection traversal shrank, and check all five scenes for regressions.

## Further candidates, ranked

1. **GPUI Fast phase logging** — 4.872290 / 4.999760 ms/attempt inclusive in the two log runs, including 3.608035 / 3.683934 inside prepaint. At [window.rs:1384](../../crates/rustui-gpui-fast/src/window.rs#L1384), format each record into one reusable buffer and issue one complete stderr write, retaining the exact record format and per-layout semantics. The phase-off control's 4.19–4.30 ms/attempt CPU reduction is an observer-overhead bound, not the measured saving of buffering. Requires a GPUI patch decision. Fix or control this observer effect before interpreting the historical ~5.3 ms as application layout work.
2. **Eager empty parse in Kit** — [highlighter.rs:648](../../crates/rustui-gpui-component/src/highlighter/highlighter.rs#L648) uses `unwrap_or(self.parser.parse("", None).unwrap())`, so the fallback expression runs even when a tree exists. Lazy fallback is a small, separate patch candidate. Tree-sitter parsing is far smaller than injection traversal here; empty-parse CPU is not separately resolved and must not be assigned all parser time. Requires an owner patch decision.
3. **RustUI editor value materialization** — [code_editor.rs:287](../../crates/rustui/src/components/advanced/code_editor.rs#L287): test reading the borrowed native `text()` and materializing only the final String, instead of `value().to_string()`'s intermediate SharedString. RustUI-owned, but exact saving is unresolved; do not expect it to remove the 9.109 ms Kit subtree. Preserve owner synchronization and IME behavior.
4. **RustUI list label rebuilding** — [list.rs:72](../../crates/rustui/src/components/data/list.rs#L72): reuse resolved plain-label storage when source/localization versions are unchanged, instead of collecting/cloning 5,000 labels at each lowering. RustUI-owned; the sampled parent is only 0.016932 ms/attempt on this hover trace, and actual lowering frequency was not counted. Preserve localization invalidation, selection and retained-row identities.
5. **GPUI Fast scroll-row signatures** — `hash_primitive` self CPU is about 0.070 / 0.072 ms/attempt on log, much smaller than instrumentation. List row hashing/scene insertion is measurable after logging; requires a GPUI patch decision. It does not justify replacing GPUI's retention/layout runtime.

## Validation and boundary

The pinned release build passed. `scripts/check.sh` passed all seven groups: **1,873 total tests, 1,866 passed and 7 ignored**. `./scripts/verify-release.sh source` passed **673 tests**. All required Samply captures validated their workload, timed window and geometry; analysis checked CPU-owner totals and kept startup/warmup outside the measured window. Source CI and review status are checked on the final PR head before the authorized merge into `p-integration`.

Only this report and `CHANGELOG.md` are committed. No runtime fix, P1 work, G2-8b work, F0 acceptance-panel run or merge to `main` is part of P0.
