# Native macOS popups assessment (N11)

## Decision and scope

**FINISH, through separate tickets of at most two hours each.** RustUI already has a low-level API and the saved AppKit implementation is recoverable, but native popups are **not production-ready in the selected source tree**. The primary integration gap is that the old helper applies v4 only to `vendor/`, while Cargo uses the owned macOS crate. Placement, dismissal and focus still need native acceptance. Keep `wip/native-macos-popups` and the existing patch versions; do not merge that old branch wholesale.

Assessment date: 2026-11-09. Production-source baseline: `788bf3a7b7a75d0d6f8d71a910b5419a2bb441b7` (S11). The separate N11 housekeeping PR changes only the standing branch-deletion rule. No code, patch, apply script, manifest, release gate or CI configuration was changed for this assessment. Commands below inspect current sources or run existing focused/source checks. Native AppKit GUI behavior is **NOT VERIFIED**; no GUI acceptance was attempted.

## 1a. Approved assets versus selected runtime

Both current macOS platform sources still reject native popups:

```console
$ rg -n 'if let WindowKind::AnchoredPopup|return Err\(PopupNotSupportedError' vendor/gpui-pre-macos-0.3.9/src/platform.rs crates/rustui-gpui-macos/src/platform.rs
vendor/gpui-pre-macos-0.3.9/src/platform.rs:775:        if let WindowKind::AnchoredPopup(_) = options.kind {
vendor/gpui-pre-macos-0.3.9/src/platform.rs:776:            return Err(PopupNotSupportedError.into());
crates/rustui-gpui-macos/src/platform.rs:775:        if let WindowKind::AnchoredPopup(_) = options.kind {
crates/rustui-gpui-macos/src/platform.rs:776:            return Err(PopupNotSupportedError.into());
```

The approved v1–v4 files are historical versions, not four applied layers. `scripts/apply_gpui_macos_anchored_popup_patch.py` selects **v4 only**, pins its patch hash, and contains pre/post SHA-256 maps for just two **vendor** files (`platform.rs`, `window.rs`). It has `--check` and `--reverse`; no separate popup JSON manifest or owned-mirror entries accompany these versions. v1–v3 are not selected by that helper. v5 remains proposed and is not selected either.

History explains the retained versions. `git log --format='%h %s' -- scripts/apply_gpui_macos_anchored_popup_patch.py patches/approved/gpui-macos-anchored-popup-v*.patch` identifies the initial runtime integration, screen-conversion revision, then bounded child shutdown:

```text
586e3bc2 fix: bound macOS popup child shutdown
acfc7e97 fix(macos): correct anchored popup screen conversion
344baa28 feat(runtime): add native anchored popup windows
```

The original helper selected v1; the current helper selects v4. The checked-in backend remains pristine by the helper's temporary-apply/restore design, rather than treating the approved directory as an automatically applied stack.

| Entry point inspected | Actual wiring |
|---|---|
| `scripts/apply_gpui_macos_anchored_popup_patch.py` | Applies v4 to the vendor backend; `--check` replays in a temporary directory without changing root sources; reverse restores pinned HEAD preimages. |
| `scripts/cargo-headstart` | On Darwin, temporarily applies that helper and restores sources on exit if this invocation applied it. It is optional. |
| `scripts/verify-release.sh macos` | Explicit helper calls at lines 134, 140 and 147: reverse-check, EXIT restoration, apply. This is transient vendor-only application after exact-source checks. |
| `scripts/verify-release.sh source` | No direct popup apply/verify-list entry. Its Python discovery includes the seven popup script tests. |
| `scripts/check.sh` | Direct stable Cargo commands; no popup helper invocation. |

Reproducible wiring inspection: `rg -n 'anchored_popup|popup_patch' scripts/apply_gpui_macos_anchored_popup_patch.py scripts/cargo-headstart scripts/verify-release.sh scripts/check.sh`; read the surrounding `source)` and `macos)` cases, rather than treating a script-name match as proof of runtime application.

```console
$ python3 scripts/apply_gpui_macos_anchored_popup_patch.py --check
verified
exit=0
$ rustup run 1.97.1 cargo metadata --format-version 1 --locked --offline > /tmp/n11-cargo-metadata.json
exit=0
```

Extracting the selected packages and the macOS dependency from that metadata gives:

```text
gpui-pre-platform 0.3.9 /Users/Abium/Developer/rustui/crates/rustui-gpui-platform/Cargo.toml
 dependency: rustui-gpui-macos path=/Users/Abium/Developer/rustui/crates/rustui-gpui-macos target=cfg(target_os = "macos")
rustui-gpui-macos 0.3.9 /Users/Abium/Developer/rustui/crates/rustui-gpui-macos/Cargo.toml
```

The owning manifest is `crates/rustui-gpui-platform/Cargo.toml`, under the macOS-target dependency `gpui_macos` (`package = "rustui-gpui-macos"`, `path = "../rustui-gpui-macos"`). Thus the helper can successfully replay v4 without enabling popups in the implementation Cargo actually compiles. Both checked-in copies are presently exact preimages. The saved WIP vendor copies are v4 postimages (next section). Existing built application binaries and other developers' checkouts were not inspected.

## 1b. Saved branch versus v4 and v5

```console
$ git rev-parse origin/wip/native-macos-popups
1ba582ee1d7f0a19728a086126055e30c624f460
$ git diff --name-status origin/main...origin/wip/native-macos-popups -- vendor/
M	vendor/gpui-pre-macos-0.3.9/src/platform.rs
M	vendor/gpui-pre-macos-0.3.9/src/window.rs
```

`git show origin/wip/native-macos-popups:<path>` was copied to temporary files and hashed with `shasum -a 256`. The resulting hashes match the helper's v4 postimage constants **exactly**:

| File | Current vendor and owned mirror SHA-256 (preimage) | WIP SHA-256 = v4 postimage |
|---|---|---|
| `src/platform.rs` | `1bf0adb67969e7389063de4d54efa70e9bb0a7ac7331a74ba74245d3bb87d801` | `6f7b88ef87d757c022a505f5b33104e727bd4e8e4ebdc6b7f3c9a5e0823c8aa6` |
| `src/window.rs` | `12d18266972087f25e876a97751af1903401ad2f9b8d9c84eab7fad9495fbd00` | `ab826e99056215713a220895f9917e5b2e39af4b4d49ccdd0fd341bb7c60daaa` |

**Classification: identical to v4 for both changed backend vendor files; different from v4 plus v5.** Main has unrelated vendor history after the old branch fork; the relevant comparison is the saved popup payload, not replacing today's entire vendor tree with the old branch.

`patches/proposed/gpui-macos-anchored-popup-v5-coordinate-fix.patch` is an incremental patch **on v4**, not a standalone backend replacement. Reading its entire diff shows three insertion-only hunks:

```diff
@@ -960,0 +961,12 @@
@@ -978,0 +991,3 @@
@@ -1165,0 +1181,18 @@
```

They add 33 lines: `popup_anchor_rect_in_view`, its call before view-to-window conversion, and `anchor_rect_conversion_respects_parent_bounds_and_flip_state`. The helper is absent from the WIP window source. The test covers bounds `(10,20,300,200)` and anchor `(5,30,10,8)`, expecting flipped origin `(15,50)` and unflipped origin `(15,182)`. An insertion-only replay in memory on the exact WIP/v4 bytes produces window SHA-256 `33a9b7bc0467c07cc9eda1b9c231e32bf0aca144afccd5b2cb1dc8fc867491ac`; no repository source was changed, and this is **not a compile result**.

The old v5 row in `docs/PATCH_TRIAGE.md` reports an exit-101 item-level `let` error and calls for a rebase. That historical compiler failure was **NOT REPRODUCED** here. Applying an incremental, zero-context v5 directly to pristine main is not valid proof against the v4-plus-v5 stack. A new ticket must pin the intended v4 preimage, regenerate contextual hunks, prove replay/reversal on vendor and owned copies, and compile/test the selected native backend before promotion.

## 1c. Existing RustUI surface and tests

| Source | Existing surface / coverage |
|---|---|
| `crates/rustui/src/runtime/window.rs:23` | Public `AnchoredPopupOptions { anchor_rect, size }`; finite geometry validation. |
| `crates/rustui/src/runtime/window.rs:32` | Public `AnchoredPopupHandle` alias of `AnyWindowHandle`. |
| `crates/rustui/src/runtime/window.rs:42` | Public `open_anchored_popup(root, environment, options, parent, cx) -> Result<AnchoredPopupHandle, String>`, exported by `runtime/mod.rs`. Builds a separate ComponentHost/Components Root using GPUI window ownership. |
| Same function | BottomLeft anchor, BottomRight gravity, flip/slide/resize flags, zero offset, **always `grab: true`**. Accepts an explicit environment. No automatic in-window fallback or typed unsupported-error policy. |
| `apps/rustui-playground/src/stories/popover_gallery.rs` | “Open native popup” / close probe, retained handle and mount/release counters. Uses click position as a 1×1 anchor and a 280×180 popup. Errors go to stderr. This is a probe, not production integration or native acceptance. |
| `crates/rustui-gpui/src/platform/test/platform.rs:557` | Test platform opens generic TestWindow instances; it does not exercise AppKit panel placement, key-window behavior or physical screens. |

Commands: `rg -n 'AnchoredPopupHandle|AnchoredPopupOptions|open_anchored_popup' crates/rustui/src apps/rustui-playground/src` and `rg -n 'anchored_popup|popup_parent' crates/rustui/src/runtime/window.rs`. No regular RustUI component currently consumes this runtime helper; the Playground is its caller. Existing in-window Popover remains separate.

Existing tests were run, without adding or changing tests:

```console
$ rustup run 1.97.1 cargo test -p rustui --lib runtime::window::tests:: --locked --offline
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1422 filtered out
exit=0
$ python3 -m unittest scripts.tests.test_gpui_macos_anchored_popup_patch -v
Ran 7 tests
OK
exit=0
```

The six GPUI tests are `opens_a_root_window_in_the_test_platform`, `closing_root_window_unmounts_the_owner`, `closing_anchored_popup_unmounts_its_root`, `popup_parent_can_open_another_popup_after_explicit_close`, `root_tab_focuses_and_activates_button`, and `initial_traversal_is_scoped_to_its_root_window` (all under `runtime::window::tests`). Only the two explicitly named popup cases test popup lifecycle; none validates AppKit behavior.

The seven Python tests cover approved-patch presence, check-mode immutability, exact/idempotent replay, exact reverse, unknown-source rejection, and patch-text assertions for parent teardown and anchor-view native-window conversion. The latter assertions do not execute native teardown or coordinate conversion. Four placement unit tests are inside v4's macOS patch, and the fifth flip/bounds test is in v5; these tests are absent from the current unpatched owned backend and were **not run**.

The probe uses a click callback, whereas `crates/rustui-gpui/src/platform/popup.rs` documents that a grabbing popup should be opened while the triggering mouse-down/input is active. That contract also leaves same-application outside clicks to the caller. These details need an explicit adapter/dismissal design, not just enabling panel creation.

Existing in-window menu behavior should be reused and preserved: `crates/rustui-gpui-component/src/menu/popup_menu.rs` binds Escape to Cancel, restores previous focus in `dismiss`, and renders nested menus with `deferred(anchored())` snapped to the current window. This does not establish cross-window native focus behavior or replace the missing native integration.

## 1d. Remaining production work

The v4 payload already creates NSPanel-backed child windows, computes an initial screen frame with flip/slide/resize adjustments, dismisses a grabbing leaf panel on key resignation, and closes child windows during parent teardown. These are useful starting points, not sufficient acceptance proof.

| Area | Present payload / precise gap | Estimate and proof boundary |
|---|---|---|
| Source integration | Vendor-only transient apply misses owned Cargo source; no paired manifest. | 2h: paired replayable patch plumbing, selected-source proof. |
| View/AppKit coordinates | v4 forwards logical top-left anchor coordinates to an NSRect without the v5 view-bounds/flip conversion. | 2h: rebase v5, contextual replay, bounds/flip/scale tests. |
| Displays and edge constraints | Anchor-center display selection, visibleFrame, flip/slide/resize exist. Negative origins, stacked displays, oversized popups and display fallback need coverage. | 2h: placement matrix; actual screens need GUI acceptance. |
| Live placement | Frame computed at creation; no stored live trigger geometry/update path. Child-window movement alone does not prove tracking after trigger layout/scroll/resize or scale changes. | 2h: define and test re-anchoring lifetime using GPUI scheduling. |
| Outside click | Resign-key closes only grabbing panels with no child windows. No new explicit same-app outside-click/nested-stack controller or monitor cleanup. | 2h: deterministic dismissal controller tests plus native click-through/activation checks. |
| Escape and focus return | No new popup Escape policy or remembered owner-element focus restoration. Generic AppKit key changes do not prove correct owner focus. | 2h: topmost-close policy, weak focus-owner recovery; keyboard dispatch tests and native focus acceptance. |
| Parent/owner teardown | Native close enumerates and closes child windows. Python checks only source text; generic test windows cover explicit close/reopen, not NSWindow callbacks. | 2h: nested close, owner removal, stale handles, weak release and unrelated-child regression coverage. |
| RustUI policy / fallback | Low-level opt-in API exists, but always grabs and stringifies errors; no integrated native-preferred/fallback policy. | 2h: preserve current API; design additive policy only with owner approval. Default in-window; fallback for unsupported native capability, surface real errors. |
| First component adapter | No production component wiring. Preserve trigger bounds, environment, bindings, owner lifetime and mouse-down contract. | 2h: one opt-in Popover adapter; other families get separate tickets. |
| Native acceptance | No actual AppKit placement/dismissal/focus evidence in these checks. | 2h owner session: multi-display/scales, activation, Escape, owner close, focus/accessibility. Inconclusive result is reported once. |

Coordinate reasoning follows the inspected source and Apple's contracts: [NSView.isFlipped](https://developer.apple.com/documentation/appkit/nsview/isflipped), [rect conversion](https://developer.apple.com/documentation/appkit/nsview/convert%28_%3Ato%3A%29-3cqqt), and [NSScreen](https://developer.apple.com/documentation/appkit/nsscreen). Distinguish parent-window logical coordinates, view bounds, window coordinates and screen coordinates; do not assume a universal flipped origin or scale factor.

| Can run deterministically in GPUI test windows / fixtures | Requires a real macOS GUI session |
|---|---|
| API geometry/error/fallback policy, explicit open/close/reopen, owner unmount, weak release, key-action controller behavior, environment/binding retention. | AppKit key/resign event order, same-app and other-app outside clicks, native child-window teardown, actual focus return and accessibility window transitions. |
| Pure placement matrix and replay/hash checks; selected-backend compilation separately. | Actual monitor arrangements, visible-frame constraints, Retina/mixed scales, Spaces and activation. |

A TestWindow passing lifecycle tests cannot certify NSPanel integration. Production readiness requires both deterministic coverage and owner native acceptance. No new public API signature is approved by this assessment.

## 1e. Benefit and candidate adopters

| RustUI family | Prospective benefit / constraint |
|---|---|
| Menu/context menu | Menus can extend outside a short or narrow parent window; requires keyboard selection, nested dismissal and focus restoration. |
| Popover | Inspector/help/detail panels avoid parent-window clipping while retaining trigger anchoring and owner lifetime. First adapter candidate. |
| Picker / ComboBox | Long option lists can use available screen space; selection/input focus must survive separate-window ownership. |
| DatePicker / ColorPicker | Calendar/color panels can extend beyond the parent; adopt only after the common popup policy works. |
| Tooltip | Hints can avoid clipping near window edges, but need a passive/non-grabbing mode; the present RustUI helper always grabs. |

These are prospective gains. Normal RustUI controls currently keep their in-window implementations; this assessment enables none of them.

## Recommendation: FINISH in bounded tickets

Budget **ten tickets × at most two hours = approximately 20 hours**, including owner acceptance. Estimates are planning limits, not claims that all native edge cases are solved. Stop/split if a ticket needs more work. Do not resume all ten as one ticket.

1. **N11a — Align native popup patch with owned Cargo sources (2h).** New proposed patch, paired vendor/owned manifest pre/post hashes, `--check`/`--reverse` apply script, script tests and correct release lists. Prove reverse/reapply in a disposable worktree. Remove reliance on transient vendor-only build mutation as part of the separately authorized integration change.
2. **N11b — Rebase coordinate fix on exact popup baseline (2h).** Contextual v5 successor, flipped/unflipped bounds and logical/scale coverage, native backend compile proof.
3. **N11c — Screen and edge placement matrix (2h).** Negative/stacked screens, visible frames, anchors/gravity, oversized panels and fallback display selection.
4. **N11d — Live anchor tracking (2h).** Trigger layout/scroll, parent resize/move and display/scale changes without losing popup identity.
5. **N11e — Outside-click and nested dismissal (2h).** Same-app/other-app activation, nested stack order, click-through policy and observer cleanup.
6. **N11f — Escape and owner focus recovery (2h).** Topmost popup close, editing semantics and weak owner focus return.
7. **N11g — Parent/owner teardown and release (2h).** Nested children, stale probe handles, weak ownership and unrelated child-window safety.
8. **N11h — Opt-in policy and in-window fallback (2h).** Preserve the existing low-level API; seek owner approval for any additive public signature. Include passive/grabbing distinction and error classification.
9. **N11i — First opt-in Popover adapter (2h).** Real trigger bounds, inherited environment, state ownership, reopen and fallback tests. Menus, pickers and tooltips require later family-sized tickets; they are not included in this first adapter estimate.
10. **N11j — Owner AppKit acceptance (2h).** Real screen/activation/focus/teardown/accessibility matrix. Production-ready only after this passes; report inconclusive native checks once.

Every GPUI vendor or owned-source change must originate in a versioned `patches/proposed/` patch with pre/post manifest and an apply script supporting `--check`/`--reverse`. Those source files change only by that script. Promotion requires proof and owner approval under AGENTS.md; N11 is assessment authorization, not promotion authorization. No `.cargo-checksum.json` edits or dependency-version changes. Retain upstream compatibility and mirror equality throughout.

## Housekeeping and validation

The sole remaining local stash was not reachable from existing archive tags. It is now permanently tagged and pushed, without dropping it:

```console
$ git stash list --format='%gd %H %gs'
stash@{0} 62cffca786ddce5258a7862d2ce6e4d275326621 On g10-small-gaps: G10 preserved part 4 for owner-requested rebase
$ git ls-remote --tags origin refs/tags/archive/stash-20261109-1
62cffca786ddce5258a7862d2ce6e4d275326621	refs/tags/archive/stash-20261109-1
```

Each candidate remote tip was checked with `git merge-base --is-ancestor <tip> origin/main` (exit 0) and matched against `git ls-remote --heads origin` before deletion. Deleted merged branches: `g10-small-gaps`, `g11-aliases`, `p0-patch-triage`, `p1-promote-two`, `p2-rebase-promote`, `p2b-slider`, `r15-disabled-controls`, `r15b-dropdown-recovery`, `r20-async-tasks`, `r26-theme-reload-tests`, `roadmap-stages`, `rule-patch-only`, `s11-stash-review`, `t9r-tools-review`. `main` and every `wip/*` were excluded. The separate AGENTS-only rule PR [#53](https://github.com/abouzoubaa/rustui/pull/53) merged as `ae83d288d55cd7e6f36b716505b7a52aacdd717e`. Its two-parent merge and rule commit were verified on fetched origin/main before deleting remote `rule-delete-merged-branches`. The assessment branch follows the same verified-merge-before-deletion rule.

Both the housekeeping branch and the assessment branch passed the same required checks in this session. Assessment commands and results:

```console
$ scripts/check.sh > /tmp/n11-assessment-check.log 2>&1; echo "exit=$?"
exit=0
$ ./scripts/verify-release.sh source > /tmp/n11-assessment-source.log 2>&1; echo "exit=$?"
exit=0
```

`check.sh`: **1,814 passed, 0 failed, 4 existing ignored** across the workspace library suites. Source release gate: **647 Python script tests passed**, plus its other checks. Existing focused tests above also passed. No new tests were added for this read-only assessment, and no product capability/status change is claimed.
