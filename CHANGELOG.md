# Changelog

## Unreleased
- F0-7: completed automated GPUI Fast matrices and benchmarks; added the owner macOS acceptance checklist.
- F0-6: refreshed Fast dependency notices, tightened owned-source verification, and covered candidate tooling in the CI cache key.
- F0-5: added no-forced-refresh stale-rendering tests for owner, binding, environment/theme, list, menu, text input, grid edits, and transitions; all pass on the current base and Fast candidate.
- F0-4i: verified the existing Fast native-popup candidates with cumulative placement, nested-dismissal, and parent-teardown proofs.
- F0-4h: copied the syntax/Diff Kit 0.9 patches into the Fast candidate stack; JSON, provider-refresh, and Diff highlighter proofs pass.
- F0-4g: copied the toolbar RTL Kit 0.9 patch into the Fast candidate stack; cumulative visual-order navigation proof passes.
- F0-4f: copied the input/text Kit 0.9 candidates into the Fast stack; placeholder color and editor-node validation proofs pass.
- F0-4e: copied the calendar/DatePicker Kit 0.9 patch family into the Fast candidate stack; cumulative localization, keyboard, RTL, accessibility and date-limit proofs pass.
- F0-4d: copied the slider/resize Kit 0.9 patch family into the Fast candidate stack; cumulative SetValue, geometry, and resize proofs pass.
- F0-4c: copied the tree/list/grid Kit 0.9 patch family unchanged into the Fast candidate stack; cumulative tree, list and table proofs pass.
- F0-4b: copied the menu/select patch family unchanged into the Fast candidate stack; cumulative Kit 0.9 menu proofs pass.
- F0-4a: added GPUI Fast accessibility serialization for disabled state, bounds and labelled-by IDs; missing disabled/grid state remains recorded in F0 findings.
- F0-3: wired the GPUI Fast 0.1.14 candidate as the sole GPUI core and rebased core, grid accessibility, and native popup patches into the candidate stack.
- F0-2: moved inactive Kit 0.8 patch history out of the active approval directory.
- F0-1: added current-base long-list, editor-typing, idle-draw, and 1,000-row performance baselines.
- Added the GPUI Fast evaluation plan and measurable adoption gates.
- Added F0 to evaluate GPUI Fast before Stage B; Stage F now depends on that decision.

## 0.4.0 — 2026-11-13
- A3-7: completed Kit 0.9.1 acceptance gates and recorded redraw performance; Stage F follow-up tracks the forced-draw regression.
- A3-6: verified owned mirrors, dependency notices, CI cache inputs, and local plus fresh Git consumers for Kit 0.9.1.
- A3: profiled Kit 0.9 redraw behavior; accepted the measured result under the 9.0 ms 0.4.0 limit and recorded the regression for Stage F.
- CI: retired the activated Kit 0.8 fixture gates and free runner disk after Kit 0.9 verification.
- A3-4i: rebased anchored native popup and child teardown candidates for Kit 0.9.1; cumulative compile and popup proofs pass.
- A3-4c: staged tree/list and delegated grid accessibility candidates with passing cumulative Kit 0.9 proofs.
- A3: isolate A2/A3 GPUI fixture compilation caches from the workspace target directory.
- A3-4a: rebased modal focus, tracked-dialog focus, and sheet close accessibility patches for GPUI Kit 0.9.1; provider-first cumulative proofs pass.
- A3-4b: rebased popup-menu disabled recovery and staged the menu/select RTL candidate family for GPUI Kit 0.9.1.
- A3-3: confirmed no Kit 0.9.1 patch is proven fully obsolete; retained all three partial behavior deltas with passing RustUI regression checks.
- A3-2: added the inert GPUI Kit 0.9 core/macOS candidate stack, Rust 1.98 compatibility delta, compile gate, and core regression proofs.
- A3-1: added the hash-pinned GPUI Kit 0.9.1 / gpui-pre 0.3.12 source fixture and disposable lock-graph inventory.
- Reordered post-A2 roadmap work: A3 and RustUI 0.4.0 precede Stage B, crates.io follows Stage B, and the NP-7 macOS crate test is parked until A3.
- Added the crates.io distribution plan, renamed GPUI package map, and bounded publishing ticket sequence.
- NP-7: removed screen-capture-only manifest dependencies and recorded the remaining Rust 1.98.0 `map_or_default` test blocker.
- NP-10: added the owner checklist for native macOS popup acceptance.

## 0.3.0 — 2026-11-11

- NP-9: added an opt-in native Popover adapter with owner environment propagation, reopen behavior, and in-window fallback on unsupported platforms.
- NP-8: added typed native-popup errors and an explicit unsupported-platform decision for the existing in-window Popover fallback.
- NP-7: parent teardown closes only owned GPUI popup children and preserves unrelated child windows.
- NP-6: native popup Escape dismissal restores weak owner focus while allowing popup content to consume Escape.
- NP-5: nested popup dismissal is leaf-first, and anchored popups close when their visible trigger is removed.
- NP-4: anchored native popups follow trigger layout and scrolling, and reposition after parent bounds or display changes.
- NP-3: covered negative and stacked display origins, main-screen fallback, and oversized popup resizing in deterministic geometry tests.
- NP-2: added fractional logical-coordinate coverage for anchored popup placement and verified AppKit conversion does not multiply by backing scale.
- NP-1: activated the approved macOS anchored-popup backend patch; native popups remain opt-in and the in-window default is unchanged.
- Pinned check, smoke, performance, and release scripts to the repository Rust toolchain, ignoring ambient compiler overrides.
- NP-0: refreshed native popup planning for GPUI Kit 0.8.1 and rebased the opt-in macOS candidate patch.
- A2: completed the GPUI Kit 0.8.1 / gpui-pre 0.3.10 upgrade and native Rust/JSON syntax coverage; 1,000-row forced-draw median was 8.24 ms vs 8.03 ms before.
- A2-6: refreshed the Kit 0.8.1 external-consumer lock and verified local and fresh Git consumers.
- A2-5a: Activated the GPUI Kit 0.8.1 candidate stack on the integration branch; recorded the remaining RustUI migration errors.

- Bumped the project toolchain to Rust 1.98.0 for `Option::map_or_default`; added the A3 GPUI Kit 0.9.x upgrade stage after A2.

- Reference: copied GPUI Kit v0.8.1 skills and design guides as documentation-only reference material.
- A2-4n: rebased Diff and syntax registry candidate patches for Kit 0.8; deferred runtime behavior proof to A2-7.
- A2-4m: rebased GPUI Kit 0.8 toolbar RTL layout and arrow navigation with a cumulative fixture proof.
- A2-4l: added the GPUI Kit 0.8 input placeholder color candidate and cumulative fixture proof.
- A2-4k: added Kit 0.8.1 calendar and DatePicker candidates with RTL, localization, date-limit, accessibility, and cumulative fixture proofs.
- A2-4i: rebased the slider candidate stack and added cumulative accessibility-action and endpoint-clamp proofs for GPUI Kit 0.8.1.
- A2-4j: rebased resize first-move and left-button-loss behavior, retiring the state-read delta already covered upstream.

## 0.2.0 — 2026-11-09

- Added Kit 0.8 Tree naming, row disabled state, and hierarchy-level candidates with a fixture tree test.

- Added the unchanged selectable-list semantics patch to the GPUI Kit 0.8 candidate stack.

- Added GPUI Kit 0.8 select and picker RTL popup layout candidates with a fixture proof.

- Added the A2-4d Kit 0.8 candidate menu patches with a disabled-confirm proof and cumulative compile gate.

- Improved source CI cache reuse with workspace artifact caching and stable checkout mtimes.

- Rebased Button disabled focus recovery for GPUI Kit 0.8 while preserving upstream #3606 traversal behavior.

- Moved the GPUI aria-disabled provider into the Kit 0.8 core candidate stack and recorded patch API dependencies.

- Added the Sheet close-button accessibility label patch to the GPUI Kit 0.8 candidate stack.

- Fixed the GPUI Kit 0.8.1 candidate bidi layout and tracked-dialog compile blockers; added an optional Base/Component compile gate.

- Added candidate GPUI Kit 0.8.1 modal focus recovery and tracked dialog closing patches.

- Added an inert-focus-scope patch candidate rebased and tested against the GPUI Kit 0.8.1 fixture.

- Added an inert GPUI Kit 0.8 core patch candidate with a pinned, reversible stack check.

- Added a hash-pinned GPUI Kit 0.8.1 source harness and staged deferred-platform patches for A2-7.

- Reclassified all approved GPUI patches against 0.8.1 behavior, separating upstream-fixed, partial, and still-needed changes.

- Added a source-verified GPUI Kit 0.8.1 upgrade plan, patch compatibility matrix and bounded migration sequence.

- Clarified Stage A assessment scope and scheduled native popup implementation and crates.io publishing after the Stage A2 upgrade.

- Added Stage A2 for the GPUI Kit upgrade and ecosystem integration guidance for Stages C, E, F, K and M.

- Assessed native macOS popup patch integration, existing coverage and bounded completion work; preserved remaining stash history and cleaned merged branches.

- Reviewed and permanently archived old stash/presync/T11 sources; no recovery tickets remain, and native popup review stays pending.

- Added roadmap stages A–N, release targets, and two-hour ticket planning.

- Converted gallery includes to modules, colocated overlay lifecycle tests, added described file acceptance, and linked editable-grid cells to their accessibility headers.

- Verified live theme auto-reload, stop, visible invalid-update recovery and owner release with deterministic GPUI-window tests.

- Reviewed framework tools R22–R26, verified native preview/composition and explicit theme reload paths, and identified deferred adapter and automatic-reload coverage work.

- Audited owner async task cancellation, restart, errors, re-render stability and release with a deterministic GPUI-window matrix.

- Completed dropdown recovery coverage and live popup-menu disabled semantics, selection recovery and re-enable behavior.

- Audited disabled controls with a shared GPUI-window matrix, fixed focus traversal and disabled semantics, and deferred seven cases to R15b.

- Switched daily Cargo aliases to stable defaults and reconsidered roadmap item 3, moving crates.io publishing into Phase 1.

- Promoted slider accessibility labels and value actions with bound-value regression coverage.

- Updated accessibility patch proposals and promotion status.

- Promoted focus-trap accessibility and disabled popup-menu confirmation fixes.

- Triaged proposed patches against the current source.

- Removed retired coordination scripts and refreshed third-party notices.

- Added the post-0.1.0 roadmap.

## 0.1.0 — 2026-11-08

- Added registered story rendering coverage and a packaged Playground smoke check.

- Cached theme style resolution and tuned the release profile for redraw performance.

- Updated release documentation and defined the Phase C MVP plan.

- Measured the proposed quad-sort V2 candidate against the baseline.

- T11c: Added RustUI-owned GPUI Base and Component sources and the patched block source.

- T11b: Added RustUI-owned macOS GPUI platform crates with source-copy verification.
- T11a: Added the RustUI-owned GPUI core crate while preserving the pinned upstream source and attribution.

- Stable Rust 1.97.1 is the daily compiler; the Headstart script remains optional.
- Removed retired development-process instructions from repository documentation.
- T3: archived retired planning documents; native macOS is first, with Web/localization/accessibility deferred and Windows/Linux last.
- Playground packaging now refuses an existing app bundle unless --replace is supplied.

- Split the compiled preview gallery into focused modules without changing behavior.
- Split graph view into model, validation, rendering, and interaction modules without changing behavior.
- Added GPUI test-window coverage for Popover and Menu lifecycle behavior.
- Added ignored Dialog and Sheet tests; focus recovery and the strict background-focus guard remain unresolved pending an incomplete proposal.
- Promoted Dialog and Sheet focus recovery with active GPUI test-window coverage.
- Added validation and validity bindings for text, secure, and search fields, with a Playground story.
- Added the editable DataGrid design for owner approval.
- Added typed editable DataGrid cells with validation and basic navigation.
- Add stable view-only sorting to EditableDataGrid.
- Add editable grid navigation, conflict recovery, and accessibility metadata.
- Add typed theme tokens and navigation surface capabilities.
- Add keyed appear and disappear transitions using GPUI springs and inert focus scopes.
- Fix editable grid Tab and Shift-Tab focus exit at its edges.
- Expose validation descriptions and editable grid read-only, sort, and header semantics.
- Add filtered Finder file drops and a Playground example.
- Add a reproducible owner-update performance baseline command.
- Rebased DataGrid validation and accessibility metadata for GPUI Kit 0.8.
- Preserve disabled accessibility on GPUI Kit 0.8 editor-owned input nodes.
- Added the GPUI Kit 0.9.x upgrade plan.
- A3-4d: rebase Slider accessibility and endpoint behavior for Kit 0.9; retain tested resize interactions.
- A3-4e: carry calendar and DatePicker accessibility, localization, RTL, and date-limit deltas to the Kit 0.9 candidate.
- A3-4f: rebase input validation descriptions and placeholder color for GPUI Kit 0.9.1.
- A3-4g: rebase Toolbar RTL layout and keyboard traversal for GPUI Kit 0.9.1.
- A3-4h: carry runtime syntax registry support and syntax dependencies to Kit 0.9.1.
