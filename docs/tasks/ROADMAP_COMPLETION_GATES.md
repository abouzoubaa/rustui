# RustUI roadmap completion gates

Completion map updated 2026-10-19. This is not itself an implementation or acceptance claim. The latest roadmap evidence includes M73 indeterminate Progress, M80/M82 collection observations, and the bounded P3D cache comparison. The isolated quad-sort candidate is still under evaluation; this report assumes neither its promotion nor a performance win. The chronological milestone text in `docs/IMPLEMENTATION_ROADMAP.md` contains older pending labels, so check each task's latest evidence before closing it.

## Source work still required

| Priority | Capability and smallest next delivery | Live source boundary | Completion gate |
| --- | --- | --- | --- |
| 1 | **R3 collection acceptance limits:** render-observed Table precedence is selected; List/Table/Tree synchronization and stable-ID `KeyedList` are implemented. `Collection` also has bounded native mutation/restoration evidence. | `crates/rustui/src/components/data/{list,table,tree,keyed_list,collection}.rs`; `docs/tasks/R3A_COLLECTION_SYNC.md`, `R3B_KEYED_LIST.md`, and M80/M82/M84 evidence. | M75/M76 and M80/M82/M84 cover bounded cases, not full acceptance. Remaining limits include full keyboard/screen-reader/RTL/platform/performance coverage and broader collection contracts. The A→B→A Table ordering limit is accepted by Contract A, not a reproduced defect; do not implement Contract B without mutation provenance. |
| 2 | **C1 Dialog and Sheet acceptance:** trigger-driven RustUI wrappers and public-API Playground stories are implemented. M112 confirms Dialog rendering and Confirm/Escape routing; M119 adds explicit literal/localized accessible naming to the semantic Dialog body node. M119 native macOS accessibility-tree evidence exposes the named `dialog` node. M120 adds explicit literal/localized accessible naming to Sheet body semantics. Focus and screen-reader acceptance remain open. The upstream Sheet host role and close-button name remain unavailable. Drawer remains a separate, unimplemented contract. | `crates/rustui/src/components/overlay/{dialog,sheet}.rs`; `apps/rustui-playground/src/stories/{dialog_gallery,sheet_gallery}.rs`; `docs/tasks/{DIALOG_PLAYGROUND_STORY,SHEET_PLAYGROUND_STORY}.md`; M110/M112/M119/M120 evidence. | Dialog and Sheet expose literal/localized accessible-name builders and name their semantic content nodes; verified focus entry, containment/restoration, screen-reader behavior, and theme/locale remain open. Sheet host/close semantics and cross-platform acceptance remain open. Do not count bounded component checks as full overlay coverage. |
| 3 | **C1 data and navigation breadth:** typed/keyed `Collection` and controlled `NavigationSplitView` are implemented; `DataGrid` and `Outline` remain planned candidates. | `crates/rustui/src/components/data/{collection,data_grid,outline}.rs`; `crates/rustui/src/components/navigation/navigation_split.rs`; M78 and M80/M82/M84 evidence in `docs/IMPLEMENTATION_ROADMAP.md`. | Do not label `Collection` or `NavigationSplitView` as stubs. Further DataGrid/Outline work needs a concrete use case and API contract. NavigationSplitView source uses `ResizableState`-backed `SplitView` and Sidebar Up/Down/Home/End/Enter handlers; M91 adds an optional tree accessible-name builder. Native resize-drag and keyboard acceptance, locale and full AX acceptance, direct release instrumentation, and adaptive single-column collapse remain open; adaptive collapse is not implemented. |
| 4 | **D1 tooling bridge:** the versioned descriptor schema/codec (D1a) is already implemented and focused-tested. D1b, a local compiled preview host, needs an architecture decision before implementation. | `crates/rustui/src/preview/{model,schema,codec}.rs` and public `rustui::preview` exports; `crates/rustui/src/theme/loader.rs::serialize_theme` remains theme-document-only; `apps/rustui-playground/src/lib.rs::preview` remains an application panel. | Decide signal-only versus payload-bearing preview events, then validate a public-API-only native preview host with property edits, action binding, owner-safe lifecycle and stale-generation rejection. UI Studio integration needs its own boundary; arbitrary Rust closures/state are not serialized. |
| 5 | **Remaining framework breadth by approved contracts:** complete semantic accessibility/navigation, transform/interaction and animation transition behavior only where a concrete product use case and native semantics are defined. | `crates/rustui/src/accessibility/navigation.rs` has no separate runtime; `SemanticsRole::Navigation` uses pinned AccessKit. Keyboard traversal/focus policy and `modifiers/{transform,interaction}.rs`, `animation/{animator,timing,transition}.rs` remain planned. `accessibility/semantics.rs`, `modifiers/{frame,background,border,shadow,opacity,padding}.rs`, and owner-scoped tasks already provide narrower implemented behavior. | Each new capability needs a public contract, one Playground story, lifecycle/keyboard/AX behavior, and a source-level check that it uses GPUI ownership and scheduling. Do not build a second reactive or rendering runtime to fill a scaffold. |

These are capability priorities, not an instruction to implement every placeholder. Other scaffold-only advanced modules, such as graph/node editors, need concrete requirements before becoming framework completion blockers. Conversely, Stage 9 preview/serialization and Stage 8 missing presentation behavior cannot be marked complete through more checks of existing controls.

## Evidence gates for implemented slices

- **Native and accessibility:** Retained inputs, navigation, FocusScope, CodeEditor, layout, decorators, drag/drop, shadows, tasks and M73 Progress have bounded observations, not full keyboard, screen-reader, platform or pixel acceptance. Sheet body semantics are now named, while the upstream surface role and close-button name remain missing (`target/headstart/evidence/sheet-accessibility-rustui-20261019.json`). DatePicker's popup button lacks an accessible name. The native macOS probe changed Slider AX value 35→70 with `AXErrorSuccess` and confirmed 70 persisted after a Toggle-triggered parent rerender (`target/headstart/evidence/slider-accessibility-native-20261019.json`); screen-reader announcement and VoiceOver acceptance remain pending. Focus removal, shortcut precedence, nested hover/drag behavior, and M73 reduced-motion behavior also remain open or unverified. Keep observations tied to exact story/build artifacts rather than a blanket framework pass.
- **Playground coverage:** The canonical app must exercise each newly implemented public family with state/property controls, locale and appearance changes, removal/restoration, and visible verification status. `rustui-lab` remains a compatibility launcher; it is not a second implementation target. Source-only catalog entries and planned theme/locale metadata do not prove runtime support.
- **Performance:** P3D measured one opt-level-0 cache workload with AX and drag disabled; it did not establish frame/GPU speed or a general regression threshold. Preserve raw provenance, add calibrated representative workloads and native visual/interaction checks, then decide whether an optimization improves its intended case without breaking the accepted fallback. The isolated quad-sort candidate requires its own correctness, production-build, visual and paired measurement review before any promotion.
- **Build and regression:** For each completed source slice, run the approved `./scripts/cargo-headstart` path with the pinned lock/vendor inputs, focused tests and relevant Playground build. Record actual results and warnings. A compile pass alone is not GUI or AX acceptance; no checks were run for this report.

## Decisions and dependency gates

1. **R3b precedence:** render-observed Table precedence is selected. Its unseen A→B→A limitation is documented and accepted. Mutation-aware precedence would require mutation provenance and possibly a separately approved versioned upstream patch; never edit the pinned vendor directly.
2. **D1 preview host boundary:** D1a's descriptor schema/codec is implemented. D1b's in-process compiled preview host and signal-only versus payload-bearing event contract await a user decision. Theme JSON export and the current Playground preview do not settle it.
3. **Asset/parser inputs:** Icon and Spinner artwork needs an approved asset source; CodeEditor highlighting/parser features need their own dependency and API decision. Current blank artwork and plain editing must remain labeled as such.
4. **P3/P4 upstream work:** approved candidate inputs, patch provenance and promotion decision precede upstream replacement. GPUI Fast, advanced RGPE GPU work and native composition require separate phase approval and compatible fallback evidence; keep GPUI Fast disabled meanwhile.
5. **X1 platform expansion:** choose each target platform and adapter scope explicitly, then collect build, native interaction and AX evidence there. macOS observations do not establish another platform.

The next source delivery should be selected from the remaining incomplete contracts after the current R3 and async-task audits. Dialog is no longer a missing source implementation; its GUI/native acceptance remains open. Close each capability with source, public API, Playground and bounded native evidence, then move to the next missing capability rather than repeating checks of already covered examples.

## Update 2026-10-19 — Dialog and Sheet

Milestone 94 records bounded Dialog interaction and visible rendering in the
canonical Playground. Accessibility role/focus reporting, theme/locale switching,
and broader retention remain open. Milestone 95 implements the first trigger-
opened RustUI Sheet adapter and passes source/package checks; its GUI story has
not been exercised. Drawer remains unspecified and is not represented by Sheet.

## Update 2026-10-19 — Sheet native check

Milestone 97 confirms the new Sheet story's right-edge rendering, title/body, and
Escape dismissal. Its accessibility snapshot omitted the title and Sheet
semantics; full accessibility, theme, locale, and platform acceptance remain
open. The disposable app outputs were removed after the check.


## Update 2026-10-19 — P3D 32-leaf comparison

M115 completed the planned 32-leaf native process comparison after fixing the
probe's missing macOS 27 theme activation. All 12 runs passed workload
correctness with no observed AX activation or native drag. Median paired
cached/uncached attached CPU ratio was 0.0729; maximum-sampled RSS ratio was
0.9873. This is workload-specific process evidence, not a frame/GPU speed
claim or visual acceptance. Raw plan and summary:
`target/headstart/performance/cache-comparison-p3d-32-20261019-c-plan.json` and
`target/headstart/performance/cache-comparison-p3d-32-20261019-c-summary.json`.
