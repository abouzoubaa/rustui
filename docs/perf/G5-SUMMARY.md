# G5 closeout

**ADOPT** means inclusion in the explicit `RUSTUI_G5_PROFILE=1` arm. Existing
defaults stay in place. The profile leaves display sync enabled and excludes
all diagnostic probes. Explicit legacy benchmark switches retain their existing
entry points. Setting the profile to 1 enables the complete ADOPT set. **KEEP OPT-IN** marks stand-alone experiments or diagnostics.

RustUI's packaged minimum is macOS 13. The legacy column describes the
code path. The forced-legacy controls in [G5-2](G5-2-frame-sources.md)
ran on a newer OS. The Swift reference and ScreenCaptureKit observers
build with a macOS 14 deployment target. The production Playground
keeps its macOS 13 minimum.

| Switch | What it does | Evidence | Recommendation | Below macOS 14 |
| --- | --- | --- | --- | --- |
| `GPUI_FAST_IMMEDIATE_PRESENT` | Presents demand immediately when idle | [#171](https://github.com/abouzoubaa/rustui/pull/171) owner accepted, CPU INCONCLUSIVE; [#175](https://github.com/abouzoubaa/rustui/pull/175) native scheduling follow-up | ADOPT | Immediate path with CVDisplayLink fallback |
| `GPUI_FAST_IMMEDIATE_HOLD_MS` | Controls input-burst hold duration | [#171](https://github.com/abouzoubaa/rustui/pull/171) 250 ms burst policy accepted, CPU INCONCLUSIVE | KEEP OPT-IN | Legacy scheduling path; existing duration unchanged |
| `GPUI_FAST_IMMEDIATE_TRANSACTION` | Uses transaction-style present for the test arm | [#171](https://github.com/abouzoubaa/rustui/pull/171) comparison dropped after shared-clock control | KEEP OPT-IN | Existing transaction support on legacy renderer |
| `GPUI_FAST_VIEW_DISPLAY_LINK` | Uses the NSView display-link arm | [#172](https://github.com/abouzoubaa/rustui/pull/172) hover CPU REGRESSION, cadence below target | REJECTED | NSView display link unavailable; legacy link |
| `GPUI_FAST_FRAME_AHEAD` | Controls the experimental Ahead arm | [#172](https://github.com/abouzoubaa/rustui/pull/172) discarded after reordering and hover latency increase | REJECTED | No Ahead arm enabled |
| `GPUI_FAST_FORCE_LEGACY_LINK` | Forces the CVDisplayLink path for testing | [#172](https://github.com/abouzoubaa/rustui/pull/172) correctness checks passed with documented acquisition outliers | KEEP OPT-IN | CVDisplayLink is already the fallback |
| `GPUI_FAST_DRAWABLE_RESERVE_TWO` | Reserved an extra drawable slot in the experiment | [#173](https://github.com/abouzoubaa/rustui/pull/173) 76–78 versions/s, CPU regressed and an acquisition still exceeded 2 ms | REJECTED | Experimental branch arm removed |
| `GPUI_FAST_IMMEDIATE_COALESCE` | Admits at most one immediate frame per refresh per surface | [#173](https://github.com/abouzoubaa/rustui/pull/173) motion CPU GAIN in tested bundle, typing INCONCLUSIVE | ADOPT | Applies per surface on legacy |
| `GPUI_FAST_IMMEDIATE_ANY_CHANGE` | Routes outside timer or async demand into immediate presentation | [#173](https://github.com/abouzoubaa/rustui/pull/173) append latency improved, CPU INCONCLUSIVE; [#175](https://github.com/abouzoubaa/rustui/pull/175) wake guards | ADOPT | Uses legacy capacity and wake guards |
| `RUSTUI_DRAW_FIRST` | Presents native text before binding and colour updates | [#173](https://github.com/abouzoubaa/rustui/pull/173) accepted typing latency, CPU INCONCLUSIVE | ADOPT | Editor path applies on legacy |
| `GPUI_FAST_NATIVE_SCHEDULER` | Queues immediate drawing after AppKit event return | [#175](https://github.com/abouzoubaa/rustui/pull/175) native input correctness; hover and drag GAIN in F1 | ADOPT | Cocoa post-event path applies on legacy |
| `GPUI_FAST_INPUT_SCOPED_DEMAND` | Serves AppKit input demand with the post-event immediate frame | [#175](https://github.com/abouzoubaa/rustui/pull/175) typing median passed, CPU INCONCLUSIVE | ADOPT | Cocoa demand path applies on legacy |
| `GPUI_FAST_TRACE_WAKES` | Logs immediate wake causes and watchdog events | [#175](https://github.com/abouzoubaa/rustui/pull/175) diagnostic | KEEP OPT-IN | Cocoa diagnostic available |
| `GPUI_FAST_LATE_DRAWABLE` | Prepares eligible rendering before drawable acquisition | [#176](https://github.com/abouzoubaa/rustui/pull/176) P improved, CPU INCONCLUSIVE, P absolute targets missed | ADOPT | Uses same guarded Metal path |
| `RUSTUI_G5_TRIM_EDITOR` | Reuses editor fold and layout bookkeeping | [#176](https://github.com/abouzoubaa/rustui/pull/176) P improved, CPU INCONCLUSIVE, P absolute targets missed | ADOPT | Same editor path |
| `GPUI_FAST_NO_DISPLAY_SYNC` | Tests unsynchronized windowed Metal presentation | [#177](https://github.com/abouzoubaa/rustui/pull/177) common-clock comparison; visual and power confirmation pending | PENDING OWNER SESSION | Windowed Metal test; fullscreen and transactions stay synced |
| `RUSTUI_G5_INCREMENTAL_EDITOR` | Tests additional wrap and fold reuse | [#178](https://github.com/abouzoubaa/rustui/pull/178) no measurable gain; Kit already splices wrap rows | REJECTED | Experiment stays off |
| `RUSTUI_G5_P_BREAKDOWN` | Records per-phase handler and rendering timestamps | [#179](https://github.com/abouzoubaa/rustui/pull/179) diagnosis only | KEEP OPT-IN | Diagnostic can run against legacy source |
| `RUSTUI_G5_KEEP_WARM` | Pins panel rate for four refreshes or 250 ms | [#180](https://github.com/abouzoubaa/rustui/pull/180) no miss-rate gain; CPU REGRESSION | REJECTED | Unsupported; helper logs and leaves link inactive |
| `RUSTUI_MAIN_LAG` | Samples main run-loop timer lateness | [#180](https://github.com/abouzoubaa/rustui/pull/180) diagnosis only | KEEP OPT-IN | CFRunLoop diagnostic available |
| `RUSTUI_G5_PREPAINT_REUSE` | Reuses shaped visible editor lines | [#181](https://github.com/abouzoubaa/rustui/pull/181) 97.3% reuse on workload, CPU INCONCLUSIVE; absolute targets missed | ADOPT | Same editor cache |
| `RUSTUI_G5_PREPAINT_VERIFY` | Checks cached layout against full layout | [#181](https://github.com/abouzoubaa/rustui/pull/181) oracle diagnostics | KEEP OPT-IN | Same layout check |
| `RUSTUI_G5_L6_TRACE` | Logs native handler detail | [#181](https://github.com/abouzoubaa/rustui/pull/181) diagnostic | KEEP OPT-IN | Native diagnostic available |
| `RUSTUI_G5_GPU_DENSITY` | Logs primitive density and covered pixels | [#181](https://github.com/abouzoubaa/rustui/pull/181) diagnostic | KEEP OPT-IN | Metal diagnostic available |
| `RUSTUI_G5_L7_SINGLE_LINE` | Selects the matched Lab single-line control | [#182](https://github.com/abouzoubaa/rustui/pull/182) matched-content diagnostic | KEEP OPT-IN | Native runner control |
| `RUSTUI_G5_L7_MATCH_EDITOR` | Selects the matched Lab document timing setup | [#182](https://github.com/abouzoubaa/rustui/pull/182) matched-content diagnostic | KEEP OPT-IN | Native runner control |
| `RUSTUI_G5_L7_EXPORT_DOCUMENT` | Exports the matching Lab document | [#182](https://github.com/abouzoubaa/rustui/pull/182) fixture control | KEEP OPT-IN | File output |
| `RUSTUI_G5_L7_DOCUMENT` | Loads the matching AppKit file | [#182](https://github.com/abouzoubaa/rustui/pull/182) matched-content diagnostic | KEEP OPT-IN | AppKit runner control |
| `RUSTUI_G5_L7_FONT` | Selects the AppKit font family | [#182](https://github.com/abouzoubaa/rustui/pull/182) fixture control | KEEP OPT-IN | AppKit runner control |
| `RUSTUI_G5_L7_FONT_SIZE` | Selects the AppKit font size | [#182](https://github.com/abouzoubaa/rustui/pull/182) fixture control | KEEP OPT-IN | AppKit runner control |
| `RUSTUI_G5_L8_CPU` | Records process and Mach-thread CPU | [#183](https://github.com/abouzoubaa/rustui/pull/183) diagnosis only | KEEP OPT-IN | Mach diagnostic available |
| `RUSTUI_G5_L8_IDLE` | Selects the focused or unfocused idle capture | [#183](https://github.com/abouzoubaa/rustui/pull/183) diagnosis only | KEEP OPT-IN | Cocoa runner control |
| `RUSTUI_G5_L8_WAIT_STACKS` | Samples blocked handler stacks | [#183](https://github.com/abouzoubaa/rustui/pull/183) diagnosis only | KEEP OPT-IN | Mach diagnostic available |
| `RUSTUI_G5_L9_LOG` | Logs injection predicates, paths and elapsed time | [#184](https://github.com/abouzoubaa/rustui/pull/184) diagnosis only | KEEP OPT-IN | Same highlighter diagnostic |
| `RUSTUI_G5_L9_SCOPED` | Reuses pinned Rust injection layers and scopes queries | [#184](https://github.com/abouzoubaa/rustui/pull/184) CPU GAIN; median 6.871 ms/key | ADOPT | Same highlighter path |
| `RUSTUI_HL_FASTPATH` | Enables PX-3’s pinned Rust empty-injection proof | [#168](https://github.com/abouzoubaa/rustui/pull/168) GAIN; [#170](https://github.com/abouzoubaa/rustui/pull/170) draw timing diagnosed; [#184](https://github.com/abouzoubaa/rustui/pull/184) draw-first restoration | ADOPT | Same highlighter path |
| `RUSTUI_FRAME_LOG=buffered` | Defers frame-recorder writes | [#165](https://github.com/abouzoubaa/rustui/pull/165) recorder observer fix | KEEP OPT-IN | Same buffered recorder |
| `RUSTUI_FRAME_PROFILE` | Includes CPU phase records | [#165](https://github.com/abouzoubaa/rustui/pull/165) observer comparison | KEEP OPT-IN | Same phase probe |
| `RUSTUI_G0_LATENCY` | Enables version-to-display measurement | [#166](https://github.com/abouzoubaa/rustui/pull/166) native presented-time proof | KEEP OPT-IN | Same Metal time probe |
| `RUSTUI_G0_HOVER_PIXELS` | Selects the two-row pixel oracle | [#166](https://github.com/abouzoubaa/rustui/pull/166) pixel proof | KEEP OPT-IN | Same Lab runner |
| `RUSTUI_G0_TYPING_HZ` | Selects Lab typing rate | [#166](https://github.com/abouzoubaa/rustui/pull/166) benchmark control | KEEP OPT-IN | Same Lab runner |
| `RUSTUI_LATENCY_BREAKDOWN` | Enables frame and drawable timing markers | [#167](https://github.com/abouzoubaa/rustui/pull/167) diagnostic | KEEP OPT-IN | Metal markers; link fields follow active source |
| `RUSTUI_L1_NATIVE` | Tracks Cocoa input stamps and version correspondence | [#174](https://github.com/abouzoubaa/rustui/pull/174) diagnostic; [#175](https://github.com/abouzoubaa/rustui/pull/175) native delivery proof | KEEP OPT-IN | Cocoa diagnostic available |
| `RUSTUI_L1_PHYSICAL` | Uses real X and Backspace instead of posted keys | [#174](https://github.com/abouzoubaa/rustui/pull/174) owner measurement control | KEEP OPT-IN | Cocoa physical input available |
| `RUSTUI_L1_TARGET_HOVER` | Addresses hover NSEvents to the test window | [#175](https://github.com/abouzoubaa/rustui/pull/175) runner control | KEEP OPT-IN | Cocoa runner control |
| `RUSTUI_L1_SPARSE` | Posts isolated sparse typing in the bare reference | [#174](https://github.com/abouzoubaa/rustui/pull/174) ProMotion experiment control | KEEP OPT-IN | Native runner control |
| `RUSTUI_L1_CAPTURE_DIR` | Selects the external pixel observer rendezvous directory | [#174](https://github.com/abouzoubaa/rustui/pull/174) runner control | KEEP OPT-IN | Bundled capture helper requires macOS 14 |
| `RUSTUI_L1_KEY_SOCKET` | Connects the authorized native key source | [#174](https://github.com/abouzoubaa/rustui/pull/174) runner control | KEEP OPT-IN | Cocoa runner control |
| `RUSTUI_L1_LAB` | Identifies the Lab native recorder target | [#174](https://github.com/abouzoubaa/rustui/pull/174) runner control | KEEP OPT-IN | Cocoa runner control |
| `RUSTUI_G5_REFRESH` | Enables passive reference cadence observation | [#174](https://github.com/abouzoubaa/rustui/pull/174) ProMotion control | KEEP OPT-IN | NSView passive probe unavailable; the runtime frame source keeps its legacy behavior |
| `RUSTUI_G5_PIXEL_CAPTURE` | Records and decodes the native version marker | [#171](https://github.com/abouzoubaa/rustui/pull/171) shared-clock comparison; [#174](https://github.com/abouzoubaa/rustui/pull/174) diagnostic | KEEP OPT-IN | Bundled capture helper requires macOS 14 |
| `RUSTUI_G5_FULLSCREEN` | Selects the native fullscreen control | [#171](https://github.com/abouzoubaa/rustui/pull/171) experimental control | KEEP OPT-IN | Native fullscreen control |
| `RUSTUI_G5_ASYNC_WAKE` | Exercises deferred wake restoration in the reference | [#171](https://github.com/abouzoubaa/rustui/pull/171) correctness control | KEEP OPT-IN | Native reference control |
| `RUSTUI_G5_OVERLAY` | Adds a GPUI composition surface in the reference | [#172](https://github.com/abouzoubaa/rustui/pull/172) overlay correctness control | KEEP OPT-IN | Legacy composition path |
| `RUSTUI_G5_SCENE` | Chooses a native workload | [#174](https://github.com/abouzoubaa/rustui/pull/174) runner control | KEEP OPT-IN | Same Lab runner |
| `RUSTUI_G5_INPUT_HZ` | Sets the bare reference input cadence | [#171](https://github.com/abouzoubaa/rustui/pull/171) runner control | KEEP OPT-IN | Same native runner |
| `RUSTUI_G5_WARMUP_S` | Sets the bare reference warmup | [#171](https://github.com/abouzoubaa/rustui/pull/171) runner control | KEEP OPT-IN | Same native runner |
| `RUSTUI_G5_TRANSACTION` | Selects transaction presentation in standalone Metal | [#171](https://github.com/abouzoubaa/rustui/pull/171) reference control | KEEP OPT-IN | Native reference control |
| `RUSTUI_G5_REFERENCE` | Marks reference launch for instrumentation | [#171](https://github.com/abouzoubaa/rustui/pull/171) runner control | KEEP OPT-IN | Runtime API availability applies |
| `RUSTUI_G5_SET` | Selects benchmark sets | [#171](https://github.com/abouzoubaa/rustui/pull/171) runner control | KEEP OPT-IN | Shell option |
| `RUSTUI_G5_RATES` | Selects benchmark rates | [#171](https://github.com/abouzoubaa/rustui/pull/171) runner control | KEEP OPT-IN | Shell option |
| `RUSTUI_G5_FLOOR_10` and `RUSTUI_G5_FLOOR_30` | Supply the measured control floor for slot analysis | [#171](https://github.com/abouzoubaa/rustui/pull/171) runner control | KEEP OPT-IN | Shell option |
| `RUSTUI_G5_BACKGROUND_LIMIT` | Selects the benchmark background CPU limit | [#171](https://github.com/abouzoubaa/rustui/pull/171) recorded loaded-machine exception | KEEP OPT-IN | Shell option |
| `MTL_HUD_ENABLED` | Shows the Metal HUD for the fullscreen reference | [#171](https://github.com/abouzoubaa/rustui/pull/171) Direct-mode control | KEEP OPT-IN | System HUD support applies |
| `MTL_HUD_LOG_ENABLED` | Logs Metal HUD observations in the reference | [#171](https://github.com/abouzoubaa/rustui/pull/171) diagnostic | KEEP OPT-IN | System HUD support applies |
| `RUSTUI_F0_ACCEPTANCE` | Opens the native acceptance Playground | [#173](https://github.com/abouzoubaa/rustui/pull/173) workload control | KEEP OPT-IN | Same acceptance view |
| `RUSTUI_F0_PANEL` | Selects the historical acceptance panel | [#173](https://github.com/abouzoubaa/rustui/pull/173) diagnostic | KEEP OPT-IN | Playground control |
| `RUSTUI_F0_SCROLL` | Selects the historical acceptance scroll workload | [#173](https://github.com/abouzoubaa/rustui/pull/173) diagnostic | KEEP OPT-IN | Playground control |
| `RUSTUI_F0_PIXELS` | Enables F0 pixel diagnostics used by latency comparisons | [#179](https://github.com/abouzoubaa/rustui/pull/179) diagnostic | KEEP OPT-IN | Runner control |
| `RUSTUI_G5_C_RUST_FILE` | Loads the Rust injection corpus in Lab | G5-C: coverage measurement | KEEP OPT-IN | Same editor fixture |
| `RUSTUI_G5_PROFILE` | Enables the ADOPT runtime switches together | [Combined profile gate](#combined-sync-on-validation) | ADOPT | Per-surface guarded legacy renderer and same editor optimizations |

`RUSTUI_PX4_LOG` and `RUSTUI_G0_LOG` name record prefixes. The corresponding
activation switches are listed above. Stage G damage remains outside the G5
profile and the validation uses `RUSTUI_DAMAGE=0`.

The inventory includes parked experiments and benchmark-only switches. They stay outside the profile.

The owner-session checkout includes the rebased [L2 windowed comparison](https://github.com/abouzoubaa/rustui/pull/177)
behind its separate switch. Only the third owner arm selects it.

## Bundled grammar coverage

The shipped RustUI build enables Tree-sitter JSON and Rust. Plain text
has no grammar. The bundled grammar registration and injection queries
come from the Kit feature list and language registry.

| Bundled entry | Injection query | L9 route |
| --- | --- | --- |
| Rust | Yes, macro invocations and rules | Scoped with the pinned query and both highlighter switches active. Full query on initial load or lost mapping/index. |
| JSON | None | No injection pass. |
| Plain text | No grammar | No injection pass. |

## Rust injection corpus

The 2,000-line Rust file contains 200 ten-line blocks, each with a macro
definition and a call. The profile ran for 60 seconds after warmup
and processed 600 native X/Backspace inputs at 10/s.
Process CPU was **8.026 ms/key**. The instrumented
injection stage used 0.245 ms/key elapsed.
Of 577 logged results, 577 used scoped queries and
0 reused a proven empty result.
The full-query route ran 0 times in the measured window.
Handler → ScreenCaptureKit was 14.899 / 20.471 ms.
Corrected background CPU was 12.82% at entry
and 16.89% at exit.

The checkout also contains disabled Kit language entries. Their source
queries are listed here to show the fallback if those languages are
ever registered. They have no runtime path in this build.

- Injection query present, full-query fallback if enabled: Astro, EJS, Elixir, ERB, HTML, JavaScript, Lua, Markdown, PHP, Svelte, TypeScript and Zig.
- No injection query: Bash, C, C#, C++, CMake, CSS, Diff, Go, GraphQL, Java, JSDoc, Kotlin, Make, Markdown Inline, Proto, Python, Ruby, Scala, SQL, Swift, TOML, TSX and YAML.

## Combined sync-on validation

Native Cocoa input, direct launches, one frozen release build, display
sync on, damage off, passive link off and buffered logging. Five
matched 60-second pairs alternate off/on, on/off, off/on, on/off and
off/on. After the 5% background check, a loaded machine can settle for
up to one minute per pair. Capture-boundary loads are recorded below.
The CPU verdict uses the P rule on process CPU seconds per wall second.

| Scene | Process CPU s/s off → on | Five changes | Verdict | Handler → SCK median / p95 off → on (ms) |
| --- | ---: | --- | --- | ---: |
| typing | 0.15567 → 0.07798 | -51.34%, -49.93%, -50.87%, -49.12%, -49.74% | GAIN | 46.531 / 53.597 → 14.439 / 19.447 |
| hover | 0.38989 → 0.35534 | -10.64%, -8.30%, -8.48%, -10.00%, -8.86% | GAIN | 36.983 / 41.506 → 16.724 / 23.678 |
| drag | 0.29370 → 0.27167 | -7.00%, -8.05%, -7.86%, -6.03%, -7.62% | GAIN | 36.742 / 42.157 → 15.078 / 21.723 |
| scroll | 0.30701 → 0.29750 | -5.22%, -3.68%, -3.10%, -1.50%, -2.68% | INCONCLUSIVE | 35.303 / 40.551 → 16.172 / 21.931 |

Latency reports the median of capture medians and of capture p95s.
Rows labelled loaded were measured on a loaded machine, with corrected
background CPU percentages shown at both capture boundaries.

| Scene | Pair | Profile | Background start / end | Load | Native inputs | SCK joins | Order violations |
| --- | ---: | --- | ---: | --- | ---: | ---: | ---: |
| typing | 1 | off | 4.53% / 17.52% | loaded | 600 | 575 | 0 |
| typing | 1 | on | 3.77% / 16.86% | loaded | 600 | 596 | 0 |
| typing | 2 | off | 17.16% / 19.96% | loaded | 600 | 576 | 0 |
| typing | 2 | on | 5.32% / 29.84% | loaded | 600 | 577 | 0 |
| typing | 3 | off | 5.97% / 22.77% | loaded | 600 | 575 | 0 |
| typing | 3 | on | 15.86% / 23.02% | loaded | 600 | 577 | 0 |
| typing | 4 | off | 4.19% / 24.59% | loaded | 600 | 575 | 0 |
| typing | 4 | on | 12.55% / 16.73% | loaded | 600 | 579 | 0 |
| typing | 5 | off | 4.23% / 16.94% | loaded | 600 | 574 | 0 |
| typing | 5 | on | 3.95% / 17.21% | loaded | 600 | 578 | 0 |
| hover | 1 | off | 7.06% / 18.56% | loaded | 6015 | 5675 | 0 |
| hover | 1 | on | 4.57% / 18.84% | loaded | 6191 | 5847 | 0 |
| hover | 2 | off | 3.28% / 19.36% | loaded | 5862 | 5511 | 0 |
| hover | 2 | on | 6.06% / 18.59% | loaded | 6190 | 5861 | 0 |
| hover | 3 | off | 4.89% / 18.31% | loaded | 5754 | 5420 | 1 |
| hover | 3 | on | 4.01% / 18.59% | loaded | 5871 | 5689 | 0 |
| hover | 4 | off | 3.95% / 20.20% | loaded | 5753 | 5436 | 1 |
| hover | 4 | on | 5.74% / 18.12% | loaded | 5876 | 5676 | 0 |
| hover | 5 | off | 6.01% / 19.30% | loaded | 5742 | 5429 | 0 |
| hover | 5 | on | 4.41% / 18.96% | loaded | 5870 | 5670 | 0 |
| drag | 1 | off | 3.48% / 18.40% | loaded | 6103 | 5686 | 0 |
| drag | 1 | on | 3.71% / 17.93% | loaded | 6165 | 6008 | 0 |
| drag | 2 | off | 9.72% / 17.52% | loaded | 6101 | 5659 | 0 |
| drag | 2 | on | 4.62% / 17.81% | loaded | 6165 | 6012 | 0 |
| drag | 3 | off | 6.51% / 19.39% | loaded | 6100 | 5666 | 0 |
| drag | 3 | on | 7.59% / 17.37% | loaded | 6163 | 6006 | 0 |
| drag | 4 | off | 4.99% / 17.59% | loaded | 6084 | 5661 | 0 |
| drag | 4 | on | 4.43% / 17.86% | loaded | 6165 | 6019 | 0 |
| drag | 5 | off | 5.32% / 18.40% | loaded | 6045 | 5609 | 0 |
| drag | 5 | on | 3.71% / 29.16% | loaded | 6109 | 5947 | 0 |
| scroll | 1 | off | 25.19% / 18.16% | loaded | 6266 | 5854 | 0 |
| scroll | 1 | on | 8.11% / 17.35% | loaded | 6297 | 6079 | 0 |
| scroll | 2 | off | 5.38% / 17.97% | loaded | 6261 | 5846 | 0 |
| scroll | 2 | on | 3.89% / 17.68% | loaded | 6305 | 6083 | 0 |
| scroll | 3 | off | 3.77% / 17.94% | loaded | 6267 | 5860 | 0 |
| scroll | 3 | on | 3.78% / 19.71% | loaded | 6305 | 6092 | 0 |
| scroll | 4 | off | 2.77% / 19.12% | loaded | 6272 | 5870 | 0 |
| scroll | 4 | on | 3.52% / 20.49% | loaded | 6296 | 6057 | 0 |
| scroll | 5 | off | 10.43% / 18.05% | loaded | 6262 | 5851 | 0 |
| scroll | 5 | on | 3.28% / 19.17% | loaded | 6305 | 6090 | 0 |

The combined validation recorded 2 presentation-order violations with
profile off and 0 with profile on. The typing captures recorded 0
lost edits. The scroll workload used macOS wheel events routed to the
active Lab window and verified that the visible list offset moved.

All four workloads passed the no-REGRESSION gate.

## G5-4 motion experiments

| Switch | What it does | Evidence | Recommendation | Below macOS 14 |
| --- | --- | --- | --- | --- |
| `RUSTUI_G5_LATCH_LATEST=1` | Dispatches queued mouse motion and compatible wheel deltas immediately before building the frame | [G5-4 baseline and candidate](G5-4-latch.md) | REJECTED. Best paired p95 improvement 1.02 ms against a 2 ms gate | Cocoa queue path uses the existing CVDisplayLink fallback |
| `RUSTUI_G5_LATCH_DEADLINE=1` | Delays the motion build until the measured render budget before the Metal display-link submit deadline | [G5-4 baseline and candidate](G5-4-latch.md) | REJECTED. Callbacks missed the cutoff and no motion versions were shown in six correctness captures | No-op with an unsupported marker |

## Local verification

`scripts/check.sh` passed with 1,875 tests passing and 7 ignored across
seven groups. Candidate source verification, the latency and damage
patch stacks, the packaged Playground smoke and a profile-on release
Playground launch passed. The three-arm owner session covers visual
tearing, power and physical typing. Those owner-run captures are pending.
