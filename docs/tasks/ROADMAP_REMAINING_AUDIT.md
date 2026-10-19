# Remaining roadmap audit (2026-10-18)

## Verified updates (2026-10-19)
- **Correction — Toggle theme role:** an attempted M89 fix was based on reading the component-local `off: secondary` role without tracing the root role map. `themes/macos27.json` maps root `secondary` to `toolbar`, so Toggle's existing toolbar-background mapping was already correct. The attempted content-surface change and its test were removed; no Toggle theme defect is established by that audit.

- The opt-in Playground theme-file reload described below was already implemented as M65 in `apps/rustui-playground/src/stories/foundation_additions.rs`, with the contract in `docs/tasks/AUTO_THEME_RELOAD.md`. It polls every 350 ms through GPUI, reloads through RustUI's `ThemeFile`, and stops when disabled or unmounted. `rustfmt --edition 2024 --check apps/rustui-playground/src/stories/foundation_additions.rs` and `./scripts/cargo-headstart check -p rustui-playground --locked` passed. No new files were needed.
- A source audit found no Playground-owned theme fallback: the app depends on `rustui`, loads `themes/macos27.json`, and activates it through `rustui::theme::activate_theme`. The framework adapter applies the profile.
- The framework coverage verifier passes for all 139 GPUI Components theme-color roles and all 24 styled macOS 27 component records. `./scripts/cargo-headstart check -p rustui` passed. No GPUI-default color fallback was identified in the audited renderers.
- Remaining declared theme limitation: dark `radio_button.blend_mode = "plus-lighter"` is retained as metadata but not rendered. The pinned GPUI/GPUI Components API has no per-element blend-mode support; the light radio rule does not declare it. No approximation or vendored patch was added.
- The trigger-driven RustUI Dialog wrapper and its public-API Playground story are now implemented. The source/build milestone is separate from GUI acceptance: the dialog has not yet been run and inspected in the canonical app. See `docs/tasks/DIALOG_PLAYGROUND_STORY.md`.

## Completed bounded delivery: Playground theme-file reload (M65)

This milestone is implemented in the existing Theme and Layout story. It is
not a pending recommendation and should not be reimplemented.

### Verified implementation boundary

- `crates/rustui/src/theme/watcher.rs`: `ThemeFile::open` and
  `reload_if_changed(mode, cx)`; invalid reads/parses/activation failures leave
  the last successful document and fingerprint intact.
- `apps/rustui-playground/src/stories/foundation_additions.rs`:
  `FoundationAdditions` stores the theme file and path and already reloads it
  from `AppAction` callbacks.
- `vendor/gpui-pre-0.3.9/src/executor.rs`: `BackgroundExecutor::timer` is an
  upstream scheduling primitive available in the pinned snapshot.
- `crates/rustui/src/runtime/tasks.rs`: owner-scoped `OwnerTask` demonstrates
  GPUI background/foreground handoff, generation checks, cancellation, and
  weak-owner behavior. Its API is one-shot, so do not assume it directly
  provides a repeating watcher.

The task record reports an opt-in 350 ms GPUI polling loop, reload through
RustUI's `ThemeFile`, and stopping when disabled or unmounted. Formatting and
the locked Playground package check are recorded as passing. This is theme-data
reload, not Rust recompilation or arbitrary Rust hot reload. Native GUI
acceptance remains a separate gate unless tied to a specific recorded run.

## Other examined gaps

- **Preview:** `apps/rustui-playground/src/lib.rs::preview` already mounts
  selected story/catalog views and source examples. There is no RustUI
  `Preview`/provider API in framework or macro crates; catalog notes explicitly
  call several compositions illustrative. A new declarative preview contract
  needs a design decision before a source-level implementation can be scoped.
- **Async tasks:** `crates/rustui/src/runtime/tasks.rs` and
  `apps/rustui-playground/src/stories/owner_tasks.rs` implement and demonstrate
  owner-scoped background tasks. Unit tests cover generation checks, cancel,
  terminal cleanup, and generation non-reuse; M72 records bounded native restart,
  cancel, error, and owner-release observations. The M72 source manifest does
  not hash `runtime/tasks.rs` or directly count private registry cleanup, so that
  runtime acceptance is not source-linked. Worker interruption and external
  side-effect cancellation remain unsupported guarantees.
- **Environment/state:** `Environment` and `EnvironmentReader` are public in
  `crates/rustui/src/view.rs`; retained children receive refreshed snapshots on
  parent lowering. `docs/tasks/F1D_ENVIRONMENT.md` explicitly limits this to
  render-time sampling: no per-value subscription, interior-mutation
  observation, or cached-child invalidation. Fine-grained observation would
  change runtime scheduling/invalidation boundaries and is not a small
  independent follow-up.
- **Serialization/UI Studio:** `serialize_theme` is theme-document export only.
  General view/state serialization remains unimplemented, and the roadmap says
  the UI Studio serialization architecture choice awaits the user's response.
  Do not start that work without the boundary decision.

## Remaining approval gates for full roadmap claims

- The roadmap's P4 GPUI Fast, upstream patching, presentation/damage work, and
  advanced GPU/native composition require separate phase approval and verified
  compatibility with the pinned upstream snapshot.
- P3 candidate revisions/patch promotion require approved pipeline and candidate
  inputs; current P2 thresholds are proposed and require baseline calibration
  and approval before controlled optimization comparisons.
- UI Studio/general state serialization requires the pending serialization
  architecture choice; theme JSON export does not settle it.
- X1 platform expansion requires a per-platform scope decision and evidence on
  each target. Current native evidence is bounded and does not establish
  platform-wide acceptance.
- Icon/Spinner artwork remains gated on approval of its asset source; no new
  asset/dependency source is implied here.

## Evidence sources inspected

- `docs/IMPLEMENTATION_ROADMAP.md` (stage 9, current verified status, F2/D1/P4/X1 queues)
- `docs/tasks/T1A_THEME_RELOAD.md`, `docs/tasks/T1B_THEME_SERIALIZATION.md`
- `docs/tasks/F2_OWNER_TASKS.md`, `docs/tasks/F1D_ENVIRONMENT.md`
- `crates/rustui/src/theme/watcher.rs`, `crates/rustui/src/runtime/tasks.rs`,
  `crates/rustui/src/view.rs`
- `apps/rustui-playground/src/stories/foundation_additions.rs`,
  `apps/rustui-playground/src/stories/owner_tasks.rs`,
  `apps/rustui-playground/src/lib.rs`, `apps/rustui-playground/src/catalog.rs`
- `vendor/gpui-pre-0.3.9/src/executor.rs`

## Update 2026-10-19 — milestones 93–96

- Build provenance now records caller-declared Cargo features; the focused test
  passed. This remains metadata, not proof of historical build flags.
- Dialog received bounded canonical-Playground checks for visible rendering,
  mouse/keyboard confirm and cancel, and Tab focus indication. Its accessibility
  snapshot still lacks a dialog role/container and reports root-window focus;
  theme/locale switching and full focus semantics remain open. See
  `target/headstart/evidence/dialog-gui-acceptance-20261019.json`.
- RustUI Sheet now wraps the pinned GPUI Components side sheet with trigger,
  optional title, and content. Focused Rust tests and the locked Playground
  package check passed; native visual acceptance remains pending. Drawer stays
  undefined and is not implemented by this Sheet.
- Cache calibration now has real 1/8/32-child scenarios with matched cached and
  uncached sets. Source tests, formatting, and example build passed. No new
  measurements were run; the prior single-workload CPU/RSS result remains the
  only performance evidence.
- Sheet's bounded GUI check now confirms right-edge rendering, title/body, and
  Escape dismissal. Accessibility still omits the title and Sheet role/container;
  broader theme/locale/platform acceptance remains open. Evidence:
  `target/headstart/evidence/sheet-gui-acceptance-20261019.json`.


## Update 2026-10-19 — M98 calibrated cache scenarios

Fixed the cache example's missing active-theme setup and completed the small, medium, and large matrix. All 18 workload records passed; per-scenario CPU and RSS summaries are recorded in `docs/tasks/P3C_CACHE_CALIBRATION.md`, with the raw evidence at `target/headstart/evidence/cache-calibration-scenario-matrix-20261019-fixed.json`. This is whole-process calibration for the mixed workload, not a cached-versus-uncached speed comparison.


## Update 2026-10-19 — M99 NavigationSplitView keyboard acceptance

The canonical Playground keyboard path passed for Up/Down, Home/End, disabled-row skipping, and Return preserving the current selection. This confirms the existing public contract; no NavigationSplitView code change was needed. Evidence: `target/headstart/evidence/navigation-split-keyboard-acceptance-20261019.md`.
