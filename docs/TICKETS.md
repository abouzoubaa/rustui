# RustUI tickets

Phase A — process (in order)

## T1 — Toolchain
**Do:** Keep Headstart for daily work; run `rustup run 1.98.0 cargo check --workspace --all-targets --locked --offline` and `rustup run 1.98.0 cargo test --workspace --lib --locked --offline`; measure one clean build and one-file-change build with Headstart and stable.
**Done when:** Both stable commands pass; report the one-time timing comparison.
**Stop if:** Stable fails; report its first error without starting another ticket.

## T2 — Remove retired process references
**Do:** Remove repository documentation references to multi-session coordination, claim DB, external skills, RTK, Headroom, and the Trail block.
**Done when:** A repository search finds none of those retired instructions in documentation.
**Stop if:** A reference belongs to runtime or product behavior rather than retired process documentation; leave it and report it.

## T3 — Archive and simplify repository documentation
**Do:** Move `docs/IMPLEMENTATION_ROADMAP.md`, `docs/tasks/`, `docs/RUSTUI_PROGRESS_AUDIT_2026-10-24.md`, and `docs/FILE_REGISTRY.md` into `docs/archive/`; remove tracked evidence folders, logs, and compiler ICE reports; add ignore rules; fix README story count to 16; update `RELEASE.md` and `RELEASING.md` for native macOS first.
**Done when:** Requested documents are archived, generated evidence/logs are absent from Git, and release docs mark Web/localization/accessibility deferred and Windows/Linux last.
**Stop if:** A target file is under `vendor/` or `target/`, or an archive destination already exists; preserve it and report the conflict.

## T4 — Stable CI source check
**Do:** Add `scripts/check.sh` for stable formatting, workspace all-target check, and workspace library tests; run the source job on every push and full release verification only manually or before merge.
**Done when:** The script passes locally and the CI source job invokes it on each push.
**Stop if:** The change requires altering unrelated CI jobs or release gates.

## T5 — Separate native and Web verification
**Do:** Make `verify-release.sh` source mode call `verify_gpui_web_patch_stack.py --native-only`; add a `web-integrity` CI job with `continue-on-error: true` running the full verifier.
**Done when:** Native-only source verification and the non-blocking Web-integrity job are configured as specified.
**Stop if:** The full Web verifier would become a release-blocking job.

## T6 — Protect existing app bundles
**Do:** Change `package-app.sh` to refuse overwriting the default bundle unless `--replace` is supplied; add a focused test.
**Done when:** Existing bundles remain unchanged by default and replacement works only with the explicit flag.
**Stop if:** The test cannot use a disposable bundle path.

## T7 — Open the main pull request
**Do:** Open one PR into `main` after confirming the three README commits merge cleanly.
**Done when:** The PR is open with a clean merge result; after the owner merges it, delete stale `work/*` branches.
**Stop if:** The README commits conflict or the owner has not merged; do not delete branches early.

## T8 — Split preview gallery source
**Do:** Split `compiled_preview_gallery.rs` into a folder with one file per component family plus registry, transport, and tests.
**Done when:** Behavior is unchanged and no resulting file exceeds 1,500 lines.
**Stop if:** The split changes behavior or requires unrelated preview redesign.

## T9 — Split graph view source
**Do:** Split `graph_view.rs` into model, validation, rendering, and interaction modules.
**Done when:** Behavior is unchanged and responsibilities live in the named modules.
**Stop if:** The split requires a graph feature redesign.

## T10 — Remove or relabel planned stubs
**Do:** Remove or relabel the 11 files marked `Status: planned`; rename `state.rs` to `component.rs` and `lifecycle.rs` to `component_host.rs` where appropriate.
**Done when:** No listed stub claims implemented status without code, and module references compile.
**Stop if:** A rename would silently break a public API; report the compatibility decision needed.

## T11 — Self-contained package for Chorus
**Do:** Create RustUI-owned `rustui-gpui`, `rustui-gpui-base`, `rustui-gpui-component`, and needed platform crates from approved patches; use path dependencies, remove matching `[patch.crates-io]` entries, and re-export `pub use rustui_gpui as gpui; pub use rustui_gpui_component as gpui_component;`.
**Done when:** An external-consumer example builds without any `[patch]` section using only `rustui::gpui`.
**Stop if:** The work requires unapproved upstream version changes or loses upstream license attribution.

- T11a — Owned GPUI core: copy `vendor/gpui-pre-0.3.9` → `crates/rustui-gpui` (package `rustui-gpui`), plus a verifier that every owned copy is byte-identical to its vendor folder except `Cargo.toml`; run it from `verify-release.sh source`.
- T11b — Owned macOS platform chain: `rustui-gpui-platform`, `rustui-gpui-macos`, `rustui-gpui-apple`. Other platforms stay later.
- T11c — Owned Base and Component: `rustui-gpui-base`, `rustui-gpui-component`; plus owned copies of `cosmic-text` and `block` (or a stated reason they are not needed).
- T11d — Integration: rustui re-exports gpui/gpui_component, root `[patch]` entries removed where replaced, external consumer builds with no `[patch]` using only `rustui::gpui`, locked consumer check in source mode.
- T11c/T11d parked on wip/t11-owned-gpui (4ffcb860). Consumer builds without [patch]. Resume: add owned-copy folders to [workspace] exclude (members glob pulls them in), restrict check.sh fmt to RustUI packages, then re-run proofs. Re-copy owned crates after T13b (vendor changed).

Phase B — features (after Phase A)

## T12 — Anchored popup lifecycle
**Do:** Add GPUI test-window coverage for open, close, reopen, and owner removal; timebox to four hours, subject to the mandatory two-hour ticket stop rule.
**Done when:** All four lifecycle cases pass in the test window.
**Stop if:** The timebox or mandatory ticket stop is reached; mark deferred and report the remaining case.

## T13 — Dialog and sheet focus recovery
**Do:** Fix focus recovery after lost focus using a proposed patch only.
**Done when:** A focused test covers focus loss and recovery, and the patch remains proposed until approved.
**Stop if:** Promotion is needed; ask the owner before promoting.

## T14 — Validation API
**Do:** Add `.validate(|v| Result<(), String>)` to TextField, SecureField, and SearchField with error text and an `is_valid` binding; add a Playground story.
**Done when:** Focused tests cover valid/invalid values and the story shows the error state.
**Stop if:** Validation requires changing existing binding semantics.

## T15 — Editable DataGrid
**Do:** First write `docs/DATAGRID.md` describing typed columns, edit commit/cancel, sorting, and cell keyboard behavior; then stop for owner approval before implementation.
**Done when:** The one-page design exists and approval is received before code starts.
**Stop if:** Approval has not arrived; do not implement the grid.

- T15a — Editable DataGrid editing core: typed text columns, row identity, validation, editing and basic navigation.
- T15b — Editable DataGrid sorting: stable view-only ascending/descending/none order.
- T15c — Editable DataGrid navigation, conflict handling and accessibility details; Tab/Shift-Tab exit at grid edges.

## T16 — Theme capabilities
**Do:** Add sidebar, toolbar, and tab-bar capabilities and a typed token API in `theme/tokens.rs`.
**Done when:** Focused tests cover token lookup and each named theme capability.
**Stop if:** The API requires changing existing serialized theme compatibility.

## T17 — Appear and disappear transitions
**Do:** Add fade/slide transitions for keyed children using existing opacity/offset springs and reduced-motion behavior.
**Done when:** Focused tests cover insertion, removal, and reduced motion.
**Stop if:** A second animation or rendering runtime is required.

## T18 — Finder file drop
**Do:** Add `DropTarget::files(...)` and a Playground story for files dropped from Finder.
**Done when:** A focused test covers accepted and rejected file drops and the story demonstrates the API.
**Stop if:** Native drag behavior cannot be tested in an owned window; report once without retrying GUI automation.

## T19 — Performance baseline
**Do:** Add a one-command `owner_update_and_draw_workload` benchmark and `perf_baseline.json`; do not optimize.
**Done when:** The benchmark reports the current result against the stated ~12.2 ms per 1,000 rows and <8 ms target.
**Stop if:** Measurement setup changes production behavior or the baseline cannot be reproduced.

## T20 — Quad-sort V2 candidate
**Do:** Measure quad-sort V2 in an isolated candidate against the T19 baseline and report a numbers table.
**Done when:** The isolated candidate has comparable measurements and a clear result table.
**Stop if:** Promotion is proposed; do not promote the candidate.

Result: no meaningful difference (docs/perf/T20-quad-sort-v2.md); not promoted; closed.

## Follow-ups (not scheduled)
- See docs/ROADMAP.md for the post-0.1.0 plan.
- Parked: wip/native-macos-popups (native anchored popups, unfinished, vendor window.rs).
- Parked: wip/stash-2026-10-26 (old stash, unreviewed).
- Post-MVP: publish to crates.io (unique crate names + owned macro crate; T11 covers Git dependency only).

Phase C — MVP release
## C1 — Release docs and MVP checklist
**Do:** Remove retired claim process from release docs; define the MVP checklist. **Done when:** RELEASE.md has the checklist and no claim/ownership text.
## C2 — Resume T11 (Git consumer without [patch])
**Do:** Continue from wip/t11-owned-gpui per the T11 resume note; re-copy owned crates from current vendor/. **Done when:** external consumer builds as a Git dependency with no [patch]; all gates green.
## C3 — 1,000-row redraw under 8 ms
**Do:** Profile owner_update_and_draw_workload, then optimize RustUI code only. **Done when:** perf-baseline.sh shows the 1,000-row forced-draw median under 8 ms, no test regressions.
## C4 — Release 0.1.0
**Do:** Version bump, CHANGELOG release section, tag v0.1.0, GitHub release notes. **Done when:** every MVP checklist item is checked.
