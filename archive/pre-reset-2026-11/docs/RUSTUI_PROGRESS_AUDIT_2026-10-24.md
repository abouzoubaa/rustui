# RustUI progress audit — 2026-10-24

## Estimate

**Estimated roadmap maturity: 58% (roughly 60%; plausible range 50–65%).**

This is an audit estimate, not an official project metric. It scores the 12
roadmap stages equally and considers source implementation, recorded validation,
and remaining acceptance or approval gates. Milestone counts and changed-file
counts are not used as a denominator.

For context, source capability alone is approximately **70% implemented**.
End-to-end acceptance is closer to **40–45%** because much of the code has
focused tests or bounded native evidence but still lacks broad keyboard,
accessibility, visual, cross-platform, or performance acceptance.

| Stage | Estimate | Implemented or evidenced | Main remaining work |
|---:|---:|---|---|
| 0 — Architectural truth | 70% | Core boundaries, docs, and dependency checks exist; latest dated fix M1213 reconciles SplitView registry status. | Keep current registry and architecture statements synchronized with the newer roadmap entries. |
| 1 — Runtime ownership | 70% | GPUI application/window startup, retained ComponentHost ownership, scoped actions, and Toast behavior are implemented. | Broad native focus, lifecycle, and screen-reader acceptance. |
| 2 — State and identity | 70% | Bindings, state, identity, environment, and async-task foundations are implemented. | Full lifecycle and fine-grained observation remain limited; broad acceptance is incomplete. |
| 3 — Playground story system | 70% | Canonical Playground, catalog, reference viewer, and many interactive stories exist. | Full-app, packaged-app, browser, visual, keyboard, and accessibility acceptance. |
| 4 — First component contracts | 65% | Core components and macros are implemented; recent parser and identity regressions have focused coverage. | Broader public API, generated-code, visual, keyboard, and accessibility acceptance. |
| 5 — Typed theme resolution | 60% | Runtime theme loading/resolution and schema validation exist; Button/List theme fixes are recorded. | Renderer coverage gaps and broader visual, accessibility, and cross-platform checks. |
| 6 — Localization foundation | 60% | Locale parsing, catalogs, fallback, direction, and formatting exist for shipped catalogs. | Full locale-data strategy and broad native/browser RTL and accessibility acceptance. |
| 7 — Inputs and selection | 60% | A broad retained input/selection component set exists; Slider keyboard behavior and identity fixes are recorded. | Consistent keyboard, focus, IME, VoiceOver, visual, and cross-platform acceptance. |
| 8 — Collections and navigation | 55% | List/Table/Tree, navigation, overlays, and collection controls are implemented; M1222 corrected duplicate TreeItem semantics. | Per-row `aria-level` remains blocked on an upstream API/approved patch; broader native accessibility and platform checks remain. |
| 9 — Framework capabilities | 45% | D1a–D1c descriptor/codec, compiled preview host, and bounded Playground inspector exist; spring/opacity primitives exist. | Broader UI Studio/composition and arbitrary view/state serialization need an architecture decision; some modules remain scaffolds. |
| 10 — Measurement and optimization | 30% | Candidate intake, snapshot, source, evidence, and aggregation validators are substantial; M1258 bounds deep cleanup traversal. | Candidate execution, full build/correctness/GUI/performance evidence, independent review, approval, and promotion. |
| 11 — Platform expansion | 45% | Web/Wasm shell and bundle exist. M1249 applied the approved Web keyboard patch and recorded Space activation in a fresh browser. | Text/IME, broader keyboard navigation, browser screen-reader/accessibility, fonts/glyphs, and platform acceptance. |

The stage estimates average to approximately 58%. The ranges are more honest
than treating that rounded score as precise.

## Current implementation shape

- The RustUI framework has concrete GPUI runtime and retained component-host
  code, public view composition, layout, a broad component catalog, core state
  and binding primitives, localization, theme loading/resolution, and preview
  infrastructure.
- The Playground is the canonical app and `rustui-lab` delegates to it. Its
  stories and reference catalog are executable examples, but catalog presence
  does not establish runtime acceptance for every component.
- Around 20 source files are explicitly scaffolds or planned modules. These
  include RGPE, adapter APIs, lifecycle, animation timing/transition/animator,
  theme tokens, runtime invalidation, alignment, Outline, Drawer, and advanced
  graph/node/resizable-panel components. The existence of a module file is not
  counted as implementation.
- D1 preview work is narrower than general serialization or UI Studio: it
  supports descriptors and a local compiled preview flow, not arbitrary Rust
  view/state serialization or general composition editing.

## Current milestone and evidence limits

The main implementation roadmap's chronological entries currently end at
**M1258**. However, the completion-gates/task ledger contains later Stage 10
entries through **M1273**, including bounded candidate-input handling (M1270),
descriptor cleanup (M1271), the exact 32 MiB patch boundary (M1272), and a
public macro type-check (M1273). The two sources are out of sync, so the
milestone table above is a stage-level synthesis rather than a complete
account of every latest task record. These newer Stage 10 entries continue to
harden local evidence and candidate tooling; they do not establish an accepted
optimization candidate.

Stage 11 has advanced since the previous estimate: the approved M855 v3 Web
keyboard patch was applied through the recorded helper, and M1249 reports Space
activation verified in a fresh browser. That closes one specific interaction
gap, not browser acceptance overall.

The broad workspace test status in the roadmap is still dated 2026-10-21
(763 tests across 20 result groups). The broad all-targets check is older than
later changes. Recent milestone entries report focused checks, but this audit
did not rerun tests, builds, or GUI checks.

## Tracking and snapshot caveats

- The roadmap's per-stage summary says it was last reconciled through M1199;
  the implementation roadmap ends at M1258, while task/gate records reach
  M1273. The broad verified-status table and File Registry also trail these
  records.
- The latest live Git snapshot showed 219 tracked files modified and 741
  untracked paths. The shared worktree is changing, so these counts are a
  snapshot only and do not measure project progress.
- No single completion percentage is defined by the roadmap. This report uses
  an explicit equal-stage maturity heuristic to answer the request for an
  approximate figure; it should be refreshed when the roadmap status and
  evidence indexes are reconciled.

## Primary references

- `docs/IMPLEMENTATION_ROADMAP.md` — stage definitions, chronological milestones,
  current-milestone summary, and recorded evidence.
- `docs/tasks/ROADMAP_COMPLETION_GATES.md` — capability-level acceptance and
  decision gates.
- `docs/tasks/ROADMAP_REMAINING_AUDIT.md` — deferred boundaries and approvals.
- `docs/FILE_REGISTRY.md` — file-to-delivery mapping, currently behind the
  newest roadmap entries.
- Framework source and Playground/catalog modules under `crates/` and `apps/`.

