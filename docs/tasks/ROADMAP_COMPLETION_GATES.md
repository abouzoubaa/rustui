# RustUI roadmap completion gates

Completion map updated 2026-10-20. This is not itself an implementation or acceptance claim. The latest roadmap evidence includes M73 indeterminate Progress, M80/M82 collection observations, and the bounded P3D cache comparison. The isolated quad-sort candidate is still under evaluation; this report assumes neither its promotion nor a performance win. The chronological milestone text in `docs/IMPLEMENTATION_ROADMAP.md` contains older pending labels, so check each task's latest evidence before closing it.

## Source work still required

| Priority | Capability and smallest next delivery | Live source boundary | Completion gate |
| --- | --- | --- | --- |
| 1 | **R3 collection acceptance limits:** render-observed Table precedence is selected; List/Table/Tree synchronization and stable-ID `KeyedList` are implemented. `Collection` also has bounded native mutation/restoration evidence. | `crates/rustui/src/components/data/{list,table,tree,keyed_list,collection}.rs`; `docs/tasks/R3A_COLLECTION_SYNC.md`, `R3B_KEYED_LIST.md`, and M80/M82/M84 evidence. | M75/M76 and M80/M82/M84 cover bounded cases. M126 additionally verifies non-selectable Escape preservation in source List and KeyedList; full collection acceptance remains open. Remaining limits include full keyboard/screen-reader/RTL/platform/performance coverage and broader collection contracts. The A→B→A Table ordering limit is accepted by Contract A, not a reproduced defect; do not implement Contract B without mutation provenance. |
| 2 | **C1 Dialog and Sheet acceptance:** trigger-driven RustUI wrappers and public-API Playground stories are implemented. M112 confirms Dialog rendering and Confirm/Escape routing; M119 adds explicit literal/localized accessible naming to the semantic Dialog body node. M119 native macOS accessibility-tree evidence exposes the named `dialog` node. M120 adds explicit literal/localized accessible naming to Sheet body semantics. M127 observed bounded Dialog Tab/Shift+Tab traversal and trigger return after Escape/Return; direct child-focus ownership and screen-reader acceptance remain open. M143 verifies bounded Sheet keyboard focus containment and trigger restoration; Sheet child-focus ownership remains unverified. See `target/headstart/evidence/dialog-focus-m127-20261019.json` and `docs/tasks/DIALOG_PLAYGROUND_STORY.md`. The upstream Sheet host role and close-button name remain unavailable. Drawer remains a separate, unimplemented contract. | `crates/rustui/src/components/overlay/{dialog,sheet}.rs`; `apps/rustui-playground/src/stories/{dialog_gallery,sheet_gallery}.rs`; `docs/tasks/{DIALOG_PLAYGROUND_STORY,SHEET_PLAYGROUND_STORY}.md`; M110/M112/M119/M120/M143 evidence. | Dialog and Sheet expose literal/localized accessible-name builders and name their semantic content nodes; M127 verifies bounded Dialog keyboard traversal and trigger return; M143 verifies bounded Sheet keyboard containment/restoration. Direct Dialog/Sheet child-focus ownership, screen-reader behavior, theme/locale, RTL, and cross-platform acceptance remain open. Sheet host/close semantics remain open. Do not count bounded component checks as full overlay coverage. |
| 3 | **C1 data and navigation breadth:** typed/keyed `Collection` and controlled `NavigationSplitView` are implemented; `DataGrid` is now a named Table-role wrapper over the existing Table; `Outline` remains a planned candidate. | `crates/rustui/src/components/data/{collection,data_grid,outline}.rs`; `crates/rustui/src/components/navigation/navigation_split.rs`; M78 and M80/M82/M84 evidence in `docs/IMPLEMENTATION_ROADMAP.md`. | Do not label `Collection` or `NavigationSplitView` as stubs. Further DataGrid work needs pinned column/cell semantics; Outline needs a concrete use case and API contract. NavigationSplitView source uses `ResizableState`-backed `SplitView` and Sidebar Up/Down/Home/End/Enter handlers; M91 adds an optional tree accessible-name builder. M125 native AX checks observed the named Workspace navigation outline and Files details/Settings details containers. M137 bounded native resize-drag passed; keyboard acceptance, locale and full AX acceptance, direct release instrumentation, and adaptive single-column collapse remain open; adaptive collapse is not implemented. |
| 4 | **D1 tooling bridge:** D1a schema/codec, D1b signal-only in-process compiled preview host, and D1c schema-driven Playground inspector are implemented. D1c locked/offline all-targets check and Playground build passed. M138 confirms keyboard traversal and Export; M140 verified keyboard Import after editing JSON, including malformed-input recovery. M144 implements and natively verifies schema-derived Boolean toggles and decimal `f64` editing with schema rejection, generation-safe remount behavior, and import/export round-trip. M145 added an exact mounted-stamp guard before numeric draft mutation. M149 captured the generation-1 amount `Binding<String>`, remounted through descriptor import at generation 2, accepted `0.246813579`, replayed the old binding with `0.987` and observed false with draft/host unchanged, then accepted current-generation `0.13579` at revision 2. This deterministic replay invokes the captured binding writer; naturally queued GPUI event dispatch remains unverified. See `target/headstart/evidence/d1c-stale-draft-replay-m149-20261020.json`. Broader component catalog coverage remains open; D1b IME composition and retention without blur remain open. | crates/rustui/src/preview/{model,schema,codec,registry,host}.rs and public rustui::preview exports; apps/rustui-playground/src/stories/compiled_preview_gallery.rs; docs/tasks/{D1B_COMPILED_PREVIEW_HOST,D1C_PLAYGROUND_PREVIEW_INSPECTOR}.md. | M125 native checks verified typed edits/binding feedback, Button action dispatch, lifecycle generations, stale/type rejection and NavigationSplit labels; M128 observed caret-position retention across parent rerender plus keyboard blur/refocus; the consolidated D1b native evidence also verifies selection survival across parent rerender and typed replacement. M147 fixes source-level IME preedit loss across parent rerender when the owner value is unchanged and preserves external owner updates; native active-IME and no-blur retention remain open. See `target/headstart/evidence/d1b-ime-rerender-m147-20261020.json` and `target/headstart/evidence/d1b-preview-host-acceptance-20261019.json`; D1c supports schema-derived string, Boolean, and fractional `f64` edits and normalized descriptor import/export with preview ID retention; M140 verified keyboard Import of valid JSON and malformed-input recovery, and M144 verified typed-property round-trip and remount edits. UI Studio integration and arbitrary Rust closures/state serialization remain separate. |
| 5 | **Remaining framework breadth by approved contracts:** complete semantic accessibility/navigation, transform/interaction and animation transition behavior only where a concrete product use case and native semantics are defined. `SemanticsRole::Status` and polite/assertive AccessKit live values are now implemented with a Playground example; native announcement acceptance remains open. | `crates/rustui/src/accessibility/navigation.rs` has no separate runtime; `SemanticsRole::Navigation` uses pinned AccessKit. Keyboard traversal/focus policy and `modifiers/transform.rs` and `animation/{animator,timing,transition}.rs` remain planned; `modifiers/interaction.rs` now implements `PointerCursor`; its public API example is in `apps/rustui-playground/src/stories/supplementary_gallery.rs`, while native hover acceptance remains open. `accessibility/semantics.rs`, `modifiers/{frame,background,border,shadow,opacity,padding}.rs`, and owner-scoped tasks already provide narrower implemented behavior. | Each new capability needs a public contract, one Playground story, lifecycle/keyboard/AX behavior, and a source-level check that it uses GPUI ownership and scheduling. Do not build a second reactive or rendering runtime to fill a scaffold. |

These are capability priorities, not an instruction to implement every placeholder. Other scaffold-only advanced modules, such as graph/node editors, need concrete requirements before becoming framework completion blockers. Conversely, Stage 9 preview/serialization and Stage 8 missing presentation behavior cannot be marked complete through more checks of existing controls.

## Evidence gates for implemented slices

- **Native and accessibility:** Retained inputs, navigation, FocusScope, CodeEditor, layout, decorators, drag/drop, shadows, tasks and M73 Progress have bounded observations, not full keyboard, screen-reader, platform or pixel acceptance. FocusScope removal follows the caller-owned binding-clearing contract; automatic clearing via a lifecycle API would need a separate future design. Sheet body semantics are now named, while the upstream surface role and close-button name remain missing (`target/headstart/evidence/sheet-accessibility-rustui-20261019.json`). DatePicker's popup button lacks an accessible name. The native macOS probe changed Slider AX value 35→70 with `AXErrorSuccess` and confirmed 70 persisted after a Toggle-triggered parent rerender (`target/headstart/evidence/slider-accessibility-native-20261019.json`); screen-reader announcement and VoiceOver acceptance remain pending. Shortcut precedence, nested hover/drag behavior, and M73 reduced-motion behavior also remain open or unverified. Keep observations tied to exact story/build artifacts rather than a blanket framework pass.
- **Playground coverage:** The canonical app must exercise each newly implemented public family with state/property controls, locale and appearance changes, removal/restoration, and visible verification status. `rustui-lab` remains a compatibility launcher; it is not a second implementation target. Source-only catalog entries and planned theme/locale metadata do not prove runtime support.
- **Performance:** P3D measured one opt-level-0 cache workload with AX and drag disabled; it did not establish frame/GPU speed or a general regression threshold. Preserve raw provenance, add calibrated representative workloads and native visual/interaction checks, then decide whether an optimization improves its intended case without breaking the accepted fallback. The isolated quad-sort candidate requires its own correctness, production-build, visual and paired measurement review before any promotion.
- **Build and regression:** For each completed source slice, run the approved `./scripts/cargo-headstart` path with the pinned lock/vendor inputs, focused tests and relevant Playground build. Record actual results and warnings. A compile pass alone is not GUI or AX acceptance; no checks were run for this report.

## Decisions and dependency gates

1. **R3b precedence:** render-observed Table precedence is selected. Its unseen A→B→A limitation is documented and accepted. Mutation-aware precedence would require mutation provenance and possibly a separately approved versioned upstream patch; never edit the pinned vendor directly.
2. **D1 preview host boundary:** D1b uses the selected signal-only in-process event contract and has a source implementation. M125 bounded native checks passed for edits, actions, lifecycle, rejection probes and NavigationSplit labels; M128 observed caret-position retention across parent rerender plus keyboard blur/refocus; text selection, IME composition, and retention without blur remain open. UI Studio integration and arbitrary Rust view/state serialization remain separate boundaries.
3. **Asset/parser inputs:** Icon and Spinner artwork needs an approved asset source; CodeEditor highlighting/parser features need their own dependency and API decision. Current blank artwork and plain editing must remain labeled as such.
4. **P3/P4 upstream work:** approved candidate inputs, patch provenance and promotion decision precede upstream replacement. GPUI Fast, advanced RGPE GPU work and native composition require separate phase approval and compatible fallback evidence; keep GPUI Fast disabled meanwhile.
5. **X1 platform expansion:** choose each target platform and adapter scope explicitly, then collect build, native interaction and AX evidence there. macOS observations do not establish another platform.

Select the next source delivery from the remaining incomplete contracts in the priority table. Dialog source implementation is complete; its broader GUI/native acceptance remains open. Close a capability’s source, public API, and bounded Playground evidence before moving to the next missing capability. M155 binds Enter to the existing Tree Confirm action; it toggles selected folder expansion and does not add leaf activation. M156 verified Enter collapses the selected Archive folder in the native Playground; Return also collapses it and Right expands it. Full keyboard/focus, screen-reader, RTL, cross-platform, and broader collection acceptance remain open.

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

## Update 2026-10-19 — X1a WebAssembly compile viability

The first compile-only WebAssembly viability check did not reach RustUI source.
`./scripts/cargo-headstart check -p rustui --target wasm32-unknown-unknown --locked --offline`
failed with E0463 because the Headstart stage1 sysroot has no `core` or `std`
libraries for that target. The default rustup installation lists the target,
but the launcher selects a separate Headstart compiler. Exact output is in
`target/headstart/evidence/x1a-wasm-check-20261019.log`. This is not WebAssembly
support evidence; building that target standard library for Headstart or using
an approved compatible toolchain is required before the compile check can run.

## Update 2026-10-19 — DatePicker accessible-name API audit

The pinned GPUI Components DatePicker exposes no accessible-label builder and
constructs the focused ComboBox internally. A RustUI wrapper would name a
different accessibility node, so no source workaround was added. The required
upstream capability and native acceptance contract are recorded in
`docs/tasks/DATE_PICKER_ACCESSIBILITY.md`.


## Update 2026-10-19 — M128–M130 native follow-up

M128 verified caret insertion-position retention in the D1b TextField across a
parent-only rerender plus keyboard blur/refocus (`abcXdef`). A later native
follow-up in the consolidated D1b evidence verified selection survival across a
parent rerender and typed replacement (`abxef`). IME and retention without blur
remain open. See `target/headstart/evidence/d1b-preview-host-acceptance-20261019.json`.

M129's P3B cache-pilot Playground counters were equal at 4/4 while GPUI
accessibility was active, so the attempt is excluded from cache-hit evidence.
excluded from cache-hit evidence. M130 added bounded Picker keyboard selection
and searchable ComboBox filtering/confirm/Escape observations; full focus
traversal and VoiceOver acceptance remain open. See the corresponding
`docs/IMPLEMENTATION_ROADMAP.md` milestones and task evidence.

## Update 2026-10-19 — M136 Sheet focus return

M136 observed visible keyboard focus on the Sheet trigger, keyboard opening, Escape dismissal with the trigger focus ring retained, and immediate Return reopening. The native accessibility API still reports window focus; it does not establish child focus ownership, focus containment, or VoiceOver behavior. Screenshots were inspected live but not retained as files. See docs/tasks/SHEET_PLAYGROUND_STORY.md and target/headstart/evidence/sheet-focus-return-m136-20261019.json.


## Update 2026-10-19 — D1c native inspector follow-up

The Playground inspector and compiled TextField both displayed `RoundTrip`; export produced normalized JSON with `properties.text` equal to `RoundTrip`. Import preserved preview ID `compiled-gallery` and remounted at generation 2, revision 0. Malformed JSON was rejected while the live value and mount remained intact; a valid descriptor was re-exported afterward. The app process was confirmed absent and its disposable bundle and executable were removed. Broader lifecycle, keyboard, accessibility, theme, locale, and cross-platform acceptance remain open.


## Update 2026-10-19 — D1b selection retention

A native macOS 27 Layout > Stacks probe selected `cd` in `abcdef`, moved focus to the preceding control and back, activated the preview-parent rerender, then typed `x`. Both the compiled field and owner showed `abxef`; generation stayed 1 and the measured rerender advanced the count from 1 to 2. Selection survived that rerender. IME, VoiceOver, automated/race, and cross-platform acceptance remain open. Evidence: `target/headstart/evidence/d1b-preview-host-acceptance-20261019.json`.


## Update 2026-10-19 — M138 D1c keyboard traversal

M138 confirms Tab traversal reaches Import and Export before the JSON editor, and Return activates Export with the current live property value. The Textarea receives keyboard focus next and owns Tab for indentation; keyboard Import after editing remains open. The bounded native result and build hash are in `target/headstart/evidence/d1c-keyboard-m138-20261019.json` and `docs/IMPLEMENTATION_ROADMAP.md`.


## Update 2026-10-19 — M139 L3B retention

M139 verified separate tab/page values, selected-tab fallback and restoration, and conditional-editor bound-value retention in the canonical Playground. Cursor/undo history, focus restoration, screen-reader, theme/locale, and cross-platform acceptance remain open. See `docs/tasks/L3B_RETENTION_CONTROLS.md` and `target/headstart/evidence/l3b-retention-m139-20261019.json`.

## Updates 2026-10-19 — M140 and M141

M140 closes the D1c keyboard Import-after-edit gap with the owner-scoped `secondary-shift-enter` action; see `target/headstart/evidence/d1c-keyboard-import-m140-20261019.json`.

M141 passes the P3B no-AX native drag fallback and separate idle cache reuse conditions. A was moved onto B, local state followed A, all three child mount counts stayed at one, active-drag fallback was observed, and the idle parent update lowered no children. Broader keyboard/focus/accessibility and performance acceptance remain open. See `docs/tasks/P3B_CACHE_DRAG_PROBE.md` and `target/headstart/evidence/p3b-cache-drag-m141-20261019.json`.

## Update 2026-10-20 — M143 Sheet focus containment

The canonical macOS 27 Sheet story kept Tab traversal within its close control
and body controls, restored the trigger focus ring on Escape, and reopened on
immediate Return. Child focus ownership and VoiceOver remain unverified. The
disposable test app was terminated after UI automation stopped responding
during cleanup; its bundle and executable were moved to Trash. See
`docs/tasks/SHEET_PLAYGROUND_STORY.md` and
`target/headstart/evidence/sheet-focus-containment-m143-20261020.json`.


## Update 2026-10-20 — M146 M66 owner destruction

M146 verified removal of the whole drag-gallery child owner while a native left-button drag was held at the target. The parent reported the released owner before mouse-up; after release the app remained responsive, no stale target update or crash was observed, and restore succeeded. The source and target shared that removed owner, so delivery to a surviving independent receiver remains unverified. Evidence: `target/headstart/evidence/m66-owner-destroyed-during-drag-m146-20261020.json`.


## Update 2026-10-20 — M148 M66 target removal

M148 removed only the drop-target subtree while the native drag remained held; the gallery owner and source stayed mounted. No destination update was observed after release. Restoring the target allowed a successful retry. This bounded negative result does not trace GPUI cancellation or event dispatch. Evidence: `target/headstart/evidence/m66-target-removed-during-held-drag-m148-20261020.json`.

## Update 2026-10-20 — M149 D1c stale numeric callback replay

M149 replayed a retained generation-1 amount binding after import remounted generation 2. The stale write returned false without changing the generation-2 draft or host value; a current-generation edit then succeeded. The deterministic callback replay does not establish a naturally queued GPUI event or race. Evidence: `target/headstart/evidence/d1c-stale-draft-replay-m149-20261020.json`.


## Update 2026-10-20 — M150 and M151

M150 records bounded native source-index List Down/Escape synchronization; Home/End are outside the pinned List key bindings, and child-focus ownership remains unknown. M151 adds Sheet localized trigger/title methods and verifies one French native render of the trigger, title, and accessible dialog name. These milestones add evidence and localized Sheet support without closing broader R3, overlay accessibility, RTL, screen-reader, or cross-platform gates. Details: `target/headstart/evidence/r3-source-list-keyboard-m150-20261020.json` and `target/headstart/evidence/sheet-localized-native-m151-20261020.json`.
