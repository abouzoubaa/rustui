# Production consolidation — 2026-10-25

This is the canonical current remaining-work record. The historical entries
below are evidence leads; their old milestones and approval assertions are not
current acceptance decisions. Scope: macOS and Web/Wasm browser. No additional
platform, GPUI Fast, or advanced GPU/native composition support is claimed.

Sole writer: thread `01a172cd-8a21-73f1-82cd-62d451c71ec9`, claim
`c44f414bfbf4ef4c4f30`, branch
`codex/production-consolidation-01a172cd`, registered worktree
`/Users/Abium/.codex/worktrees/rustui-production-consolidation/rustui`.
Integration target verified from the primary checkout:
`work/m118-diagnostics-provenance` at
`567f933c76b90fde3fcdda4ff45dc660df1eb217`; it descends from `main`.
Reverify its HEAD and dirty state before integration. Do not overwrite its work.

## Ownership and preservation

- Dispatcher `rustui-roadmap-coordination-sweep` is PAUSED.
- All 11 prior claims were explicitly superseded. Ten owners acknowledged
  quiescence; the user separately authorized administrative retirement of
  Stage 8 claim `214af0307629e64d5706` after its owner declined acknowledgment.
  The app reported idle/completed turns, not terminal enums. The user explicitly
  authorized using verified retirement acknowledgments as transfer evidence;
  the claim evidence records that distinction.
- All 23 original registered worktrees and their branches remain retained.
  Exact initial inventories, tracked binary diffs, staged diffs, recent commits,
  comparisons and retirement reports are under
  `/tmp/rustui-preservation-20261025/`. Full thread tool results are outside
  checkouts at `/tmp/rustui-single-writer-thread-inventory-20261025.json` and
  the subsequent retirement inventory files.
- Coordinator branch `codex/session-claims` was integrated by fast-forward
  (25 commits). Its uncommitted 16-line coordination clarification is retained.
  Pre-existing `graft/` caches remain in the original worktrees.
- Recovered 1,103 primary paths, excluding vendor modifications and conflicting
  coordinator files; merged both Headstart launcher changes. Applied only the
  three already-approved patch sets with exact-identity helpers, retaining
  pristine checksum metadata. Proposed patches are not applied.
- Unique preview registry/codec/schema, locale, font and macro regressions were
  recovered. The Stage 10 applicator now uses descriptor-pinned, bounded,
  iterative inventory for both public and already-pinned callers.
  Older snapshots which would remove current accessibility or binding fixes
  were not substituted for the primary source. Remaining unique differences
  are retained for review. The Stage 1 Picker replacement-subscription runtime
  fix is integrated with a focused GPUI-handle lifetime regression.

## Initial requirement inventory (provisional)

There are **32 provisional release requirements** below. Only requirements
R00–R02 are closed by this audit so far: **coding 3/32; acceptance 3/32
(9.4%, provisional audit closure)**. This is a conservative closure count,
not an estimate of how much source code exists or framework maturity.
All other rows remain open until their complete contract and current evidence
are reviewed. “Present” describes recovered source, not completed acceptance.
Expand this inventory when the full contract audit identifies distinct requirements.

| ID | Stage / required capability | Current implementation and remaining release work |
| --- | --- | --- |
| R00 | 0: independence and dependency boundary | Closed: current dependency guard passes; retain zero Chorus/Allui dependency and pinned GPUI/Components versions. |
| R01 | 0: one writer and preserved integration | Closed: dispatcher paused, 11 prior claims superseded with explicit evidence, one isolated writing scope; originals retained. |
| R02 | 0: reproducible approved patch selection | Closed for the three existing approved patch sets: exact helpers apply/verify and pristine checksum manifests match. New promotions remain gated. |
| R03 | 0: architecture, registry, public documentation | Audit all exported APIs and planned/scaffold claims; reconcile stale lifecycle/animation/tooling status. |
| R04 | 1: application/window bootstrap | Present; current library checkpoint passed. Native launch/error/close and packaging acceptance remain. |
| R05 | 1: lifecycle, owner actions, subscriptions | Present; F1E/M16 native ownership acceptance open. Picker obsolete-subscription runtime fix and focused handle-lifetime test integrated; full lifecycle event-path tests remain. |
| R06 | 2: state, identity, binding, observation, environment | Present within documented GPUI/render-time contracts; audit invalidation, retention and error boundaries against F1 contracts. |
| R07 | 2: layout, geometry and animation identity | Present; current-source native geometry, RTL and reduced-motion observations remain. |
| R08 | 2/9: timing/animation API | Opacity/spring present; timing/transition scaffolds need original-authority review and bounded implementation. No second scheduler. |
| R09 | 3: Playground catalog, stories, events, source examples | Present; current library tests passed. Validate reachability, examples and visual/keyboard behavior in the canonical app. |
| R10 | 4: macro grammar and public integration tests | Present; unit checkpoint passed. Run public integration targets and matching-toolchain doctests; diagnose M69 from current artifacts. |
| R11 | 4: basic components and semantics | Present; audit source contracts and current native visual/focus/AX behavior. |
| R12 | 5: theme states, metrics and fallbacks | Static coverage passes for 139 roles/24 styled records; runtime pack validation, light/dark/locale visuals and unapplied radio blend-mode requirement remain. |
| R13 | 6: bounded locale/catalog contract | Three runtime locale records validate; current locale unit tests pass. Native/Web locale switching, fallback and RTL acceptance remain. |
| R14 | 6: full CLDR/ICU/Fluent runtime | Missing approved runtime/data strategy and dependency versions; choose cross-target strategy before adding dependencies. Bounded handwritten rules are not full coverage. |
| R15 | 7: persistent inputs, selection, disabled behavior | Present; audit disabled focus/activation/AX, replacement subscriptions and event paths. Optional upstream test-support sources are absent. |
| R16 | 7/9: text selection and active IME | Source preservation logic and delayed no-blur preview trigger present in recovered primary; native active composition and Web IME still unverified. |
| R17 | 8: collections/navigation synchronization | Present; verify keyed replacement, route recovery, keyboard/RTL and virtualization behavior with current source. |
| R18 | 8: Tree row accessibility | aria-disabled approved patch reproduced; outer-row aria-level is a prepared, exact-hash proposed upstream patch requiring approval. Native AX/VoiceOver open. |
| R19 | 8: overlays and focus containment | Present within bounded wrappers; inspect focus-trap upstream proposal and verify modal keyboard/AX/dismissal contracts. |
| R20 | 9: async ownership, cancellation, error paths | Present; audit generation/cleanup guarantees and bind native release evidence to current runtime source. |
| R21 | 9: accessibility and keyboard across public controls | Partial; semantic projection, action handling, disabled state and actual VoiceOver need current evidence on both targets. |
| R22 | 9: descriptor/schema/codec validation | Present; recovered additional registry/schema/codec tests pass in current library checkpoint. Full contract review remains. |
| R23 | 9: compiled preview/inspector | Present; inspect edit/rejection/remount/generation behavior and run current native inspector acceptance. |
| R24 | 9: structural composition editing | No selected boundary; descriptor-first proposal is prepared. Requires a real architecture choice. |
| R25 | 9: D1d external consumer adapter | No named external consumer/transport. Define host/version/action/disconnect policy before implementation. Keep RustUI independent. |
| R26 | 9: live theme/data reload | Present, distinct from Rust source hot reload; verify invalid-update recovery and owner-release cleanup. |
| R27 | 10: performance, provenance and candidate workflow | Extensive bounded tooling present; current-source GUI/performance evidence and candidate review/promotion open. No optimization speed claim or GPU phase acceptance. |
| R28 | 11: Web reproducible build/package/assets | Source tooling present; rebuild current source and bind browser observations to exact bundle manifest. |
| R29 | 11: browser keyboard/AX/IME/fonts | M855 approved patch reproduced; focused-input accessible-name patch prepared but not approved. Full semantic controls/action routing, IME, glyph and screen-reader acceptance remain. |
| R30 | Release: CI and reproducible release builds | No GitHub workflow found in recovered checkout. Establish runnable compiler/sysroot/bootstrap policy and required macOS/Web checks. |
| R31 | Release: security, licensing, performance budgets, packaging/instructions | Complete threat/error audit, dependency/license and font notices, supported-version matrix, reproducibility, package signing/distribution plan and release instructions. Publication requires approval. |

## Current executed evidence

- `./scripts/cargo-headstart metadata --locked --offline --format-version 1`
  resolved the recovered default dependency graph.
- The reused-target library checkpoint failed before tests with E0463 for
  `rustui_i18n`; shared caches were preserved.
- One isolated persistent retry:
  `CARGO_TARGET_DIR="$PWD/target/headstart/verification/production-consolidation"
  ./scripts/cargo-headstart test --workspace --lib --profile headstart --locked --offline`
  passed **1,202 tests across six binaries: 884, 9, 81, 40, 188, 0**.
  Log: `/tmp/rustui-consolidated-isolated-lib-20261025.log`.
- Coordinator unit suite: 21/21; Headstart launcher regression: 2/2, including
  matching tool-rustdoc selection, subprocess library-path restoration and caller
  override. The macro doctest command completed with 0 cases using the matching
  Headstart rustdoc; it no longer falls back to the incompatible host tool.
- Public integration checkpoint passed **43 tests across 31 binaries**, including
  M69 nested optional closures. Log:
  `/tmp/rustui-consolidated-integration-tests-20261025.log`.
- Picker replacement-subscription lifetime regression passed 1/1 (884 filtered).
  Named slots replace/drop obsolete handles while preserving other subscriptions;
  the callback also rejects events from an obsolete select entity. This uses
  ordinary GPUI Subscription handles and adds no test-support dependencies.
- Playground's direct upstream icon dependency was moved behind
  `rustui::component_assets()`. The architecture guard now rejects direct GPUI
  dependencies, including renamed optional packages, in Playground/Lab.
  Boundary suite passed 9/9; Playground library passed 188/188 after this change.
- The P3 descriptor-pinned filesystem suite passed 23/23.
- All 804 resolved packages declare SPDX license metadata; vulnerability,
  license-compatibility and packaged-notice audits are still open.
- Recovered P3K applicator suite: 63/63
  (`/tmp/rustui-consolidated-p3k-tests-20261025.log`).
- Dependency boundary, 3 runtime locale records, 11 theme planning records,
  and static theme adapter coverage passed. Planning-record checks do not
  validate runtime pack rendering.
- Proposed Tree aria-level and Web accessible-name helpers passed exact staged
  application/reversal checks; no proposed patch was applied.
- Enabling GPUI test-support during locked/offline metadata fails first on
  missing `futures-concurrency`. The pinned GPUI manifest requests optional
  `futures-concurrency 7.7.1`, `postage 0.5`, and `proptest 1`;
  these names are absent from the current lockfile/vendor. This is a distinct
  graph expansion, not restoration of an existing locked package.
- No native GUI, browser, VoiceOver, release build or performance validation has
  run in this consolidation session. Historical Playground PIDs 10507 and
  23568 were reidentified but left untouched because application ownership
  remains unresolved; reserve the GUI slot before any new validation.

## Next work

1. Finish integration of unique retained work and inspect stale approval gates
   against original human/repository authority.
2. Validate the Picker event path and full lifecycle suite; resolve the optional
   test-support graph without changing upstream revisions.
3. Run affected public integration targets, package/doc checks and stage-level
   native/browser acceptance with current provenance.
4. Resolve the prepared upstream/localization/composition decisions while
   continuing independent release work. Production readiness remains open.

---

# Historical evidence and investigation leads

## Refresh — 2026-10-24, workspace library regression suite

The locked/offline workspace library suite passed all six test binaries:
867, 9, 77, 40, 183, and 0 tests respectively (1,176 total). This provides
source-level regression evidence only; native and browser acceptance, open
architecture decisions, and candidate review/approval/promotion gates remain
separate.

```sh
./scripts/cargo-headstart test --workspace --lib --locked --offline --quiet
```

## Refresh — 2026-10-24, existing-bundle Stage 11 browser observation

- **Stage 11:** a fresh in-app browser confirmed Button pointer activation, Image-story embedded sample rendering, and navigation to the packaged `List.svg` reference. The served bundle files were timestamped 04:45 UTC, before the Menu Bar story source at 05:12 UTC; this run does not exercise M1194. The Playground accessibility snapshot still exposes only a generic image and text-entry area; Space/Tab did not establish control keyboard access. Screen-reader, network-status, broad font/glyph, and cross-browser acceptance remain open. Details: `docs/tasks/X1B_BROWSER_HOST.md`.
- **Stage 11 RTL sample:** a second fresh browser tab visibly mirrored the catalog/reference panes for Arabic, rendered Arabic coverage labels, and retained Button pointer activation. This does not establish document `lang`/`dir` metadata or comprehensive glyph coverage.

## Refresh — 2026-10-24, M1280

- **Stage 8 M1280:** horizontal SplitView retains physical Left/Right divider movement in RTL while mirroring panel order; the focused regression passed 1/1. Native accessibility and interaction acceptance remain open.

## Refresh — 2026-10-24, Stage 3 action-feedback accessibility source audit

- **M1194:** its changing menu feedback uses the existing polite `Status` live-region semantics. The locked/offline Cargo retry stopped during dependency resolution because `futures-concurrency` is missing from the vendored source. The exact focused owner-action test then passed 1/1 from a persistent prebuilt binary whose timestamp postdates the current story source; it covers generated `Action` dispatch, not the rendered popup-row event path.
- **M1205:** its focused action-feedback test now also checks the initial `Last activated: None` status value and passed 1/1 in a persistent isolated target. Native accessibility-tree and VoiceOver acceptance remain open.
- **M1210:** its focused owner-state regression passed 1/1 in an isolated persistent target after the live-region source change. This is source-level state evidence only.
- **M1187:** its focused document-action regression now checks the initial, closed, and created-document live-region payloads; it passed 1/1 in an isolated persistent target. Native accessibility-tree and VoiceOver acceptance remain open.
- **M1208:** the changing visible Button label is also the Button's accessible-name fallback and is forwarded to GPUI. No extra status region was added. This source audit does not prove a native accessibility-tree update or VoiceOver announcement.
- **M1194 callback path:** `MenuView` binds the owner action into the enabled popup row's click handler; the existing regression dispatches generated `Action` values directly and does not activate that rendered row. See `docs/tasks/M1194_STAGE3_INTERACTIVE_MENU_BAR_SIDEBAR.md`; rendered Menu acceptance remains open.
- Rendered Menu activation and native visual, keyboard, accessibility-tree, and VoiceOver acceptance remain open across these stories.

## Refresh — 2026-10-24, M1274–M1278

- **Stage 3 M1194:** the focused owner-action regression now dispatches all six File/Edit/View/Item/Window/Help menu actions through generated `Action` values and checks notification and resulting state; it passed 1/1. Rendered Menu event-path and native visual, keyboard, and accessibility acceptance remain open.
- **Stage 2 M1274:** `AnimatedOpacity` retains its initial configuration in GPUI element state for stable identities; the focused regression passed 1/1. Native animation fidelity remains unverified.
- **Stage 3 M1275:** Arabic localized coverage filtering and Button-row selection now have a focused source regression, passed 1/1. Native rendering and interaction acceptance remain separate.
- **Stage 4 M1276:** the macro parser keeps parenthesized `Option::and_then` children distinct from adjacent constructors; the public integration test passed 1/1, the full macro unit and integration targets passed 41/41, and the isolated parser suite passed 31/31. The original doctest phase failed with `E0514` under host Rust 1.97.1 `rustdoc`; a later focused package doctest run used the matching Headstart 1.101.0-dev rustdoc from `stage1-tools-bin` and a fresh persistent target, then exited successfully with 0 doctest cases. The toolchain failure is resolved; the crate has no doctest examples. No parser defect was observed.
- **Stage 10 M1277:** an injected patch-cache cleanup failure now verifies that apply-owned descriptors close before the cleanup exception propagates; the focused test passed 1/1. Candidate execution and later acceptance remain open.
- **Stage 11 M1278:** the package preflight accepts valid multiline module-script tags; the shell regression suite passed with stub build tools. Browser acceptance remains open.
- The Stage 5 `/assets/playground-sample.png` 404 was a direct URL probe, not a missing packaged asset: the Image story uses a GPUI embedded asset source and the PNG is compiled into Wasm. No packaging defect was established. Stage 5 native locale/visual acceptance remains open.

## Refresh — 2026-10-24, M1271–M1273

- **Stage 10 M1271:** `_patch_series` now closes its locally owned candidate-root descriptor when temporary-cache construction fails. The injected-failure regression passed for standalone-open and pinned-descriptor duplication; the caller's pinned descriptor remains open.
- **Stage 10 M1272:** the synthetic intake regression accepts exactly 32 MiB, verifies the digest and cache size, and rejects one extra byte. Both M1271 and M1272 are focused local tests; neither applies a patch nor runs a candidate.
- **Stage 4 M1273:** a public `HStack!` integration test covers M1211's turbofish `.then` and `.then_some` optional-child forms; the focused test passed 1/1 in both the isolated Stage 4 worktree and primary checkout. Parser behavior is unchanged.
- Aggregate temporary-cache storage, schema 3 output staging, candidate execution, downstream acceptance, review, approval, and promotion remain open under their existing gates.

## Refresh — 2026-10-24, M1264 live-child follow-up
- **Stage 10 M1264:** the dependency-boundary checker now has a real-child regression for concurrent stdout/stderr overflow, bounded failure, child termination/reaping, and deadlock avoidance. The focused regression passed 1/1 and the full focused suite passed 8/8. This supersedes the earlier fake-process-only coverage note; candidate execution and later acceptance remain open.

## Refresh — 2026-10-23, M1269–M1270
- **Stage 10 M1270:** P3K now caches verified patch bodies on disk and applies one at a time, retaining M441's exact-byte integrity while bounding resident patch inputs to one 32 MiB body. Eight focused regressions and Python bytecode compilation passed. Aggregate temporary storage and schema 3 output staging remain unbounded by this slice; candidate execution and acceptance remain open.
- **Stage 10 M1269:** P3K rollback reuses iterative P3G cleanup while preserving the original package identity and bounding enumerated entries at 100,000. The focused rollback regressions passed 4/4 and the P3G filesystem suite passed 23/23. Candidate execution and acceptance remain open.

## Refresh — 2026-10-23, M1266–M1268
- **Stage 10 M1268:** P3H caps total manifest-scan workspace entries at 100,000 in standalone and pinned paths, rejecting on the first excess entry. The reporter suite passed 35/35.
- **Stage 10 M1267:** the P2B dependency-reuse verifier bounds Cargo JSONL input at 64 MiB while preserving the source hash; oversized input fails before report output. Focused verifier tests passed 3/3.
- **Stage 10 M1266:** P3L `run_step` now incrementally captures combined validator output with a 64 MiB ceiling, terminates the process group on overflow, and refuses to save truncated P3H stdout as a report. The focused pipeline checks passed 6/6. Candidate execution and acceptance remain open.

## Refresh — 2026-10-23, M1264–M1265
- **Stage 10 M1265:** P3H candidate-source reporting now bounds combined Cargo metadata stdout/stderr at 32 MiB in standalone and pinned paths and terminates a live child on overflow; the focused reporter suite passed 40/40, and final JSON output is bounded at 32 MiB with compact invalid output rather than truncation. Candidate execution and acceptance remain open.
- **Stage 10 M1264 review limitation:** the dependency-boundary checker's focused tests passed 7/7 and a read-only review found no concrete defect, but the tests use a fake process and do not exercise live overflow termination. M1265 adds live-child overflow coverage for the separate candidate-source reporter. Candidate execution and later acceptance remain open.

## Refresh — 2026-10-23, M1253–M1264
- **Stage 10 M1264:** the dependency-boundary checker caps combined Cargo metadata stdout/stderr capture at 32 MiB and terminates the child on overflow; focused tests passed 7/7. The tests use a fake process and do not exercise live overflow termination. Candidate execution and acceptance remain open.
- **Stage 10 M1263:** P3H caps candidate Cargo.lock and Cargo config reads at 32 MiB in standalone and pinned paths; focused lock/config regressions passed 4/4. Metadata stdout/stderr capture is covered by M1264 for the dependency-boundary checker and M1265 for the candidate-source reporter; candidate execution and acceptance remain open.
- **Stage 10 M1262:** P3H bounds candidate Cargo manifest reads at 32 MiB in standalone inventory, pinned inventory, and automatic/explicit pinned workspace selection; focused regressions passed 3/3. Candidate execution and acceptance remain open.
- **Stage 10 M1261:** P3G inventory rejects same-inode file mutation during hashing by comparing descriptor and pathname versions; focused regression passed 1/1. Candidate execution and acceptance remain open.
- **Stage 10 M1260:** P3K package copying now uses iterative, bounded-descriptor traversal; the 1,100-level low-descriptor regression passed 1/1. Candidate execution and acceptance remain open.
- **Stage 10 M1259:** P3D requires observation records for failed/inconclusive attempts even without screenshots, while retaining the `not_run` exemption; focused regression passed 1/1. Candidate execution and acceptance remain open.
- **Stage 10 M1258:** P3G `PinnedDir.remove_tree` now uses iterative cleanup with a constant number of active directory descriptors independent of nesting depth, plus identity checks on descent, ascent, and root removal. The 1,100-level low-descriptor regression and focused filesystem suite passed 22/22; Python bytecode compilation passed. Candidate execution and acceptance remain open.
- **Stage 10 M1257:** P3M now requires explicit `status: "passed"` on the M276 success event; the focused aggregate-evidence suite passed 71 tests and Python bytecode compilation passed. Candidate execution and acceptance remain open.
- **Stage 10 M1256:** the P3F aggregator now enforces its cumulative 4 GiB artifact budget on the second artifact-read pass and reuses the approved-intake bytes from that pass. The focused aggregate-evidence suite passed 71 tests and Python bytecode compilation passed. Candidate execution and acceptance remain open.
- **Stage 10 M1255:** P3H pinned manifest hashing now uses constant-descriptor traversal; the 1,100-level low-descriptor test and pinned path-swap regression passed 2/2. Candidate execution and acceptance remain open.
- **Stage 10 M1254 review follow-up:** `copy_tree_from` now closes partial FD setup, rejects overlapping source/destination roots, verifies destination directory identity after creation, and retains sanitized directory modes. Focused regressions are recorded in `M1254_STAGE10_P3G_BOUND_INTAKE_COPY_DEPTH.md`; these checks do not close candidate execution or acceptance gates.
- **Stage 10:** M1253 bounds both snapshot-validator and production P3G pipeline inventory to one active directory descriptor; both 1,100-level low-descriptor regressions passed, and `_copy_source` integration passed 1/1. M1254 applies bounded-depth traversal to intake copying while retaining M1243's entry/byte streaming limits; its 1,100-level low-descriptor regression passed 1/1. M1252 rejects blank P3H enabled feature names (focused regression 1/1), M1251 binds P3K rollback cleanup to the original package identity (focused regression 1/1), and M1250 hardens `PinnedDir.remove_tree` (focused checks 12/12). Candidate execution, current-source GUI/performance evidence, independent review, approval, and promotion remain open.
- **Stage 10 M973 follow-up:** P3D visual-evidence parsing now rejects floating-point overflow in both manifests and referenced artifacts; the `1e10000` regression passed 1/1. Candidate execution and later acceptance gates remain open.
- **Stage 10 local regression follow-ups:** P3F now verifies an exact-fit aggregate evidence byte budget succeeds and a one-byte-short limit rejects; P3K now tests entry-limit enforcement during pinned inventory/copy. The P3K preparer's integrity guards also remain active under `python -O`. These add local contract coverage only; candidate execution and acceptance remain open.

# Remaining roadmap audit (2026-10-18)

## Refresh — 2026-10-23, M1247–M1249

- **Stage 10:** M1247 bounds descriptor-pinned P3K inventory/copy with P3G's 100,000-entry and 4 GiB limits; focused tests passed 2/2. Candidate execution, current-source GUI/performance evidence, review, approval, and promotion remain open.
- **Stage 11:** M1248 rejects module-script `type` prefixes such as `modulex`; the focused package regression passed. M1249 applies approved M855 v3 through the versioned helper and same-version Cargo path override; fresh-browser keyboard acceptance verified canvas tab focus and Space activation on Increment. Text entry, screen-reader/browser accessibility, navigation/assets, and font/glyph acceptance remain open.
- **Stage 10:** M1244–M1246 address local evidence-integrity, snapshot-resource, and package-preflight defects; candidate execution and browser acceptance remain separate.

## Refresh — 2026-10-23, M1244–M1246

- **Stage 10:** M1244 rejects M274 executable suffix aliases in hash-bound events; M1245 applies P3G entry/byte caps during descriptor-pinned source and destination inventory hashing. These are local evidence-integrity/resource-boundary fixes. Candidate execution, current-source GUI/performance evidence, review, approval, and promotion remain open.
- **Stage 11:** M1246 makes package preflight recognize valid case-insensitive HTML script/type attribute names and single-quoted module values while rejecting `data-type` lookalikes. This is packaging-source evidence only; fresh-browser keyboard, navigation/assets, accessibility/screen-reader, and font/glyph acceptance remain open. The M1249 now verifies the focused GPUI Space activation path; text entry/IME, browser accessibility, navigation/assets, and font/glyph acceptance remain open.

- **Stage 6:** M1231's four-letter rejection was incorrect and has been
  reverted under M1239 after checking RFC 5646 §2.1. The cross-target locale
  strategy is still undecided; full CLDR/ICU/Fluent
  coverage and runtime RTL/accessibility acceptance remain open.
- **Stage 10:** M1229 bounds P3D visual artifact reads; M1230 and M1232 close
  P3K package descriptors on preflight and patch-check setup errors; M1233
  binds every path-source file into P3G's tree hash; M1234 requires explicit
  P3M event pass status; M1235 rejects P3N PNGs without image-data bytes;
  M1236/M1237 reject unsupported Cargo dependency kinds in P3H/P3J; M1238
  binds the P3L command argv to its M274 build/check event; M1240 requires the
  P3E prepared candidate snapshot hash; M1241 bounds approved patch copies to
  32 MiB before creating the destination. M1242 rejects empty and whitespace-
  only AnimatedOpacity animation identities; M1243 enforces existing P3G tree
  entry and byte limits during candidate source copying.
  P3F's bounded record/artifact validator is complete
  for its local-integrity contract. The named candidate's
  correctness evidence is not end-to-end acceptance: current-source GUI and
  performance evidence, independent review, approval, and promotion remain
  open. Existing M582 P3D captures have a binary hash mismatch against current
  source and cannot be reused for acceptance.
- **Stage 11:** the Web/Wasm target remains selected. Browser keyboard,
  navigation/assets, accessibility tree and screen-reader output, and
  font/glyph acceptance remain open. M1249 applied approved M855 v3 and
  verified canvas keyboard focus and Space activation in a fresh browser;
  text-entry/IME and the other Stage 11 browser gates remain open.
- **Stage 8:** RustUI uses the pinned GPUI Tree's outer row semantics; per-row
  `aria-level` remains gated on an upstream API or approved patch. Native and
  browser accessibility acceptance remains open.
- **Stage 4:** M1242 rejects empty and whitespace-only `AnimatedOpacity`
  animation identities. Native animation acceptance remains open.
- **Stages 3, 5, and 7:** the sampled Playground stories, theme adapter, macro
  parser, Slider, and Stepper had no additional grounded source defect in this
  pass. Their recorded native visual, keyboard, interaction, locale-switch,
  and screen-reader acceptance remains incomplete; broader macro/runtime
  acceptance also remains open.
- **Pending design decisions:** Stage 1 lifecycle, Stage 2 timing, Stage 6
  locale strategy, and Stage 9 composition/D1d boundary. Advanced GPU/native
  composition and upstream patch promotion remain separately gated.

## Correction — Stage 9 serialization status (2026-10-23)

The older notes below predate the completed D1 work. D1a–D1c are now
implemented: the versioned descriptor schema/codec, signal-only local compiled
preview host, and bounded Playground inspector. This does not serialize
arbitrary Rust views or runtime state, and it does not complete UI Studio.
The proposal recommends a descriptor-first composition path; approval for that
broader boundary is pending. See `docs/tasks/SERIALIZATION_BRIDGE_PROPOSAL.md`.

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
- **Serialization/UI Studio:** `serialize_theme` remains theme-document export
  only. D1a–D1c provide descriptors, the compiled preview host, and the bounded
  Playground inspector; they do not provide general view/state serialization,
  composition editing, source editing, or D1d external-host integration. The
  descriptor-first proposal is available, and approval for that broader
  boundary remains pending.

## Remaining approval gates for full roadmap claims

- The roadmap's P4 GPUI Fast, upstream patching, presentation/damage work, and
  advanced GPU/native composition require separate phase approval and verified
  compatibility with the pinned upstream snapshot.
- P3 candidate revisions/patch promotion require approved pipeline and candidate
  inputs; current P2 thresholds are proposed and require baseline calibration
  and approval before controlled optimization comparisons.
- Broader UI Studio composition and general state serialization require the
  pending descriptor-first boundary decision; theme JSON export does not
  settle them.
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


## Update 2026-10-19 — M100 Menu ownership and dependency boundaries

Native acceptance confirmed an open Menu popup disappears from the accessibility tree when its RustUI owner is removed. Stale-action activation was not possible in the observed state; component-entity destruction remains unverified. See `docs/tasks/O1_MENU_DROPDOWN.md` and `target/headstart/evidence/menu-owner-removal-20261019.json`.

A read-only manifest/source audit found no confirmed Allui or Chorus dependency violation: resolved packages contain neither, Playground depends on public `rustui`, GPUI dependencies are owned by the framework crate, and Lab delegates to Playground.

## Refresh 2026-10-23 — M1222

M664 is a historical Stage 9 LazyGrid compiled-preview specimen. The current
chronological roadmap head is M1219 (Stage 10 identity-bound M276 correctness
rerun). For candidate `quad-stable-sort-v2-chain-m272-20261020`, the locked /
offline M276 harness passed 10/10 and the P3M aggregator returned `consistent`
with build and correctness passed. GUI, performance, independent review,
approval, and promotion remain open. Stage 11 Web/Wasm browser keyboard,
accessibility-tree, font/glyph, and platform acceptance remain open. Stage 9
D1d external-host integration and broader composition-editing architecture
also remain open; Stage 8 TreeItem semantics-preserving row accessibility and
native accessibility/interaction acceptance remain open. No global completion
percentage is defined by the roadmap because implementation and acceptance
gates are separate.

See `docs/tasks/M1219_STAGE10_IDENTITY_BOUND_M276_RERUN.md`, `docs/tasks/M1220_STAGE11_REBUILD_WEB_BUNDLE.md`, and `docs/tasks/M1221_STAGE10_INDEPENDENT_REVIEW.md`, and `docs/tasks/M1222_STAGE8_TREEITEM_ROW_SEMANTICS_BOUNDARY.md` and
`docs/IMPLEMENTATION_ROADMAP.md`.

## Stage 6 full locale runtime gap — 2026-10-23

`rustui-i18n` has no approved implementation path to full CLDR/ICU/Fluent
coverage with its current dependencies. The detailed audit found only
transitive ICU support crates and no Project Fluent runtime. The next step is
a cross-target locale-data/dependency decision or a host-native adapter
contract; no partial hand-coded locale rule is being presented as full
coverage. See `docs/tasks/STAGE6_FULL_LOCALE_RUNTIME_GAP_AUDIT.md`.

## Refresh 2026-10-23 — M1223

M1223 is the current recorded milestone: Stage 5 ListView reconciliation now
keeps locale-resolved row labels stable across rerenders and replaces them when
the resolved locale changes. Its focused test passed 1/1, and an independent
source review found no regression. Native visual and live locale-switch
acceptance remain open. A follow-up retained-delegate regression now verifies
that the English resolved labels survive an unchanged rerender and that the
French resolved labels replace them after a locale change; it passed 1/1.

Stage 11's M1220 bundle rebuild establishes packaging and static serving only.
M1249 later applied approved M855 v3 and verified focused Button Space
activation in a fresh browser. Selected-story pointer/assets/navigation,
broader keyboard navigation, browser accessibility and screen-reader output,
text entry/IME, and font/glyph coverage remain unverified. Stage 10's named M276
candidate has 10/10 correctness evidence and an independent review; GUI,
performance, approval, and promotion remain open. Stage 8 still lacks outer
tree-row `aria-level` support. Stage 2's timing API remains approval-gated,
and Stage 4 has no named reproducible failure after M1211. Stage 7's M1204
native keyboard and screen-reader acceptance remains open. Stage 5 still needs
live locale-switch and native visual acceptance. Stage 1 lifecycle
implementation has a separate proposal ready for review; its reserved scaffold
requires approval before implementation
(`docs/tasks/STAGE1_LIFECYCLE_API_PROPOSAL.md`). Stages 0 and 3 retain their
other recorded acceptance gaps. The Stage 6 cross-target locale strategy
remains undecided. No roadmap-wide completion percentage or evidence-based
finish date is defined.

## Stage 3 implementation update — 2026-10-23

Closed a source-level interaction gap in M1184: each illustrative Close,
Minimize, and Zoom click now marks the preview active before recording the
selection. The focused regression deactivates the preview before each click and
asserts reactivation; the focused locked/offline test passed 1/1. This does not
close the separate Stage 3 native acceptance gaps.

## Open architecture decisions — 2026-10-23

Focused decision documents are ready for the remaining proposal-gated work:

- Stage 1 lifecycle: approve retaining the current `View::on_mount` /
  `View::on_unmount` contract and GPUI entity-release boundary before aligning
  scaffold/registry documentation
  (`docs/tasks/STAGE1_LIFECYCLE_API_PROPOSAL.md`).
- Stage 2 timing: approve the proposed timing-function API, formulas, first
  consumer, and file boundary before implementation
  (`docs/tasks/STAGE2_TIMING_FUNCTION_PROPOSAL.md`).
- Stage 6 localization: choose the cross-target locale-data/runtime strategy,
  targets, data/version policy, and Fluent contract before changing dependency
  revisions or the public API
  (`docs/tasks/STAGE6_LOCALE_RUNTIME_ARCHITECTURE_DECISION.md`).
- Stage 9: choose the bounded composition-editing boundary and name the D1d
  external consumer/transport before defining remote action integration
  (`docs/tasks/STAGE9_COMPOSITION_AND_D1D_DECISION_MEMO.md`).

These are decision aids, not approvals or implementation. Stage 11 M855 v3 was
subsequently approved and applied under M1249; any new upstream patch remains
separately gated.

## Refresh 2026-10-23 — M1224–M1225

M1224 removes the Web shell's body-level `aria-busy` state so it does not block
the descendant loading live-region announcement; the focused packaging check
passed, with browser screen-reader acceptance still open. M1225 bounds M287
screenshot and accessibility-tree reads to 64 MiB before hashing or PNG
parsing; its focused oversized-artifact regression passed 1/1. This closes a
local resource-boundary defect only. Stage 10 GUI/performance acceptance,
approval, and promotion remain open.

## Refresh 2026-10-23 — M1226–M1228

Stage 10 implementation now requires P3G path-source tree hashes, rejects P3I
path packages with contradictory raw source metadata, and rechecks candidate
inventory file type/link count/identity around hashing to close a replacement
race. Each change has a focused regression passing 1/1. These strengthen local
evidence intake and consistency; they do not establish candidate build,
correctness, GUI, performance, review, approval, or promotion gates.
